use std::{
  collections::HashSet,
  marker::PhantomData,
  sync::{Arc, Mutex},
  time::Duration,
};

use ansi_term::Color::{Green, Yellow};
use miette::LabeledSpan;
use regex::Regex;
use reqwest::StatusCode;
use serde::Deserialize;
use swc_core::{
  common::{sync::Lazy, Span},
  ecma::{
    ast::{CallExpr, Expr},
    visit::{noop_visit_type, Visit, VisitWith},
  },
};
use swc_ecma_utils2::jsx::{unpack_jsx, JSXElement, JSXRuntime};
use tokio::{sync::Semaphore, task::JoinSet};

use super::{
  diagnostics::{DeferredSource, RelativeSpan},
  symbols::Symbols,
};

struct LinkCheckCollect<'a, R: JSXRuntime> {
  links: &'a mut Vec<(url::Url, Span)>,
  invalid_links: Vec<(url::ParseError, Span)>,
  jsx: PhantomData<R>,
}

#[derive(Deserialize)]
enum Link {
  #[serde(rename = "reference")]
  Reference { refuri: String },
  #[serde(rename = "target")]
  Target { refuri: String },
}

impl<R: JSXRuntime> Visit for LinkCheckCollect<'_, R> {
  noop_visit_type!();

  fn visit_call_expr(&mut self, call: &CallExpr) {
    let uri = match unpack_jsx::<R, Link>(call) {
      Ok(Link::Reference { refuri } | Link::Target { refuri }) => Some(url::Url::parse(&refuri)),
      Err(_) => None,
    };
    let Some(uri) = uri else {
      call.visit_children_with(self);
      return;
    };
    let span = call.as_arg0_span::<R>();
    match uri {
      Ok(uri) => {
        log::trace!("found link {uri} at {span:?}");
        self.links.push((uri, span));
      }
      Err(e) => {
        log::trace!("invalid link at {span:?}: {e}");
        self.invalid_links.push((e, span));
      }
    }
  }
}

#[derive(miette::Diagnostic, thiserror::Error, Debug)]
pub enum LinkDiagnostic {
  #[error("bad link")]
  #[diagnostic(severity = "error", code(sphinx::linkcheck::not_found))]
  NotFound {
    status: StatusCode,
    uri: url::Url,
    #[source_code]
    src: DeferredSource,
    #[label(collection)]
    sites: Vec<LabeledSpan>,
    #[help]
    help: Option<String>,
  },

  #[error("link permanently redirected")]
  #[diagnostic(severity = "advice", code(sphinx::linkcheck::redirected))]
  PermanentlyRedirected {
    location: url::Url,
    uri: url::Url,
    #[source_code]
    src: DeferredSource,
    #[label(collection)]
    sites: Vec<LabeledSpan>,
    #[help]
    help: Option<String>,
  },

  #[error("GitHub links may change in the future")]
  #[diagnostic(severity = "warning", code(sphinx::linkcheck::not_permalink))]
  NotPermalink {
    uri: url::Url,
    #[source_code]
    src: DeferredSource,
    #[label(collection)]
    sites: Vec<LabeledSpan>,
    #[help]
    help: Option<String>,
  },

  #[error("link may be unintentional")]
  #[diagnostic(severity = "warning", code(sphinx::linkcheck::special_address))]
  MaybeLocalHost {
    address: String,
    uri: url::Url,
    #[source_code]
    src: DeferredSource,
    #[label(collection)]
    sites: Vec<LabeledSpan>,
    #[help]
    help: Option<String>,
  },

  #[error("failed to fetch")]
  #[diagnostic(severity = "error", code(sphinx::linkcheck::failed))]
  CouldNotCheck {
    #[source]
    error: anyhow::Error,
    uri: url::Url,
    #[source_code]
    src: DeferredSource,
    #[label(collection)]
    sites: Vec<LabeledSpan>,
    #[help]
    help: Option<String>,
  },
}

impl LinkDiagnostic {
  pub fn url(&self) -> &url::Url {
    match self {
      LinkDiagnostic::NotFound { uri, .. }
      | LinkDiagnostic::PermanentlyRedirected { uri, .. }
      | LinkDiagnostic::NotPermalink { uri, .. }
      | LinkDiagnostic::CouldNotCheck { uri, .. }
      | LinkDiagnostic::MaybeLocalHost { uri, .. } => uri,
    }
  }

  pub fn to_labels<'a>(&self, spans: impl Iterator<Item = &'a RelativeSpan>) -> Vec<LabeledSpan> {
    let link = self.url();
    spans
      .map(|span| {
        let text = span.text();

        let label = if text.contains(link.as_str()) {
          match self {
            LinkDiagnostic::NotFound { status, .. } => format!("{status}"),
            LinkDiagnostic::PermanentlyRedirected { ref location, .. } => {
              format!("redirected to {location}")
            }
            LinkDiagnostic::NotPermalink { .. } => "not a permalink".to_string(),
            LinkDiagnostic::CouldNotCheck { ref error, .. } => format!("{error}"),
            LinkDiagnostic::MaybeLocalHost { .. } => "special address".to_string(),
          }
        } else {
          match self {
            LinkDiagnostic::NotFound { status, .. } => format!("{status}: {link}"),
            LinkDiagnostic::PermanentlyRedirected { ref location, .. } => {
              format!("redirected to {location}")
            }
            LinkDiagnostic::NotPermalink { .. } => format!("not a permalink: {link}"),
            LinkDiagnostic::CouldNotCheck { ref error, .. } => format!("{error}: {link}"),
            LinkDiagnostic::MaybeLocalHost { .. } => format!("special address: {link}"),
          }
        };

        span.labeled(&label)
      })
      .collect()
  }

  pub fn with_labels(&mut self, labels: Vec<LabeledSpan>) -> &mut Self {
    match self {
      LinkDiagnostic::NotFound { sites, .. }
      | LinkDiagnostic::PermanentlyRedirected { sites, .. }
      | LinkDiagnostic::NotPermalink { sites, .. }
      | LinkDiagnostic::CouldNotCheck { sites, .. }
      | LinkDiagnostic::MaybeLocalHost { sites, .. } => *sites = labels,
    }
    self
  }

  pub fn with_source(&mut self, span: Option<&RelativeSpan>) -> &mut Self {
    match self {
      LinkDiagnostic::NotFound { src, .. }
      | LinkDiagnostic::PermanentlyRedirected { src, .. }
      | LinkDiagnostic::NotPermalink { src, .. }
      | LinkDiagnostic::CouldNotCheck { src, .. }
      | LinkDiagnostic::MaybeLocalHost { src, .. } => src.set(span.map(|src| src.source())),
    }
    self
  }
}

pub fn collect_links(module: &Expr, links: &mut Vec<(url::Url, Span)>) {
  let mut collect = LinkCheckCollect {
    links,
    invalid_links: vec![],
    jsx: PhantomData::<Symbols>,
  };
  module.visit_with(&mut collect);
}

pub async fn check_links(links: Vec<url::Url>) -> anyhow::Result<Vec<LinkDiagnostic>> {
  let mut diagnostics = vec![];

  let redirects: Arc<Mutex<HashSet<url::Url>>> = Default::default();
  let redirects_mut = redirects.clone();

  let concurrency = Arc::new(Semaphore::new(8));

  let client = reqwest::Client::builder()
    .redirect(reqwest::redirect::Policy::custom(move |attempt| {
      if matches!(
        attempt.status(),
        StatusCode::MOVED_PERMANENTLY | StatusCode::PERMANENT_REDIRECT
      ) {
        redirects_mut
          .clone()
          .lock()
          .unwrap()
          .insert(attempt.url().clone());
      }
      if attempt.previous().len() > 10 {
        attempt.stop()
      } else {
        attempt.follow()
      }
    }))
    .build()?;

  let mut requests = JoinSet::from_iter(
    links
      .iter()
      .filter(|&link| {
        if link.scheme() != "http" && link.scheme() != "https" {
          log::debug!("skipping non-HTTP link {link}");
          false
        } else if let Some(host) = maybe_special(link) {
          let url_text = link.to_string();
          diagnostics.push(LinkDiagnostic::MaybeLocalHost {
            address: host,
            uri: (*link).clone(),
            src: Default::default(),
            sites: Default::default(),
            help: Some(format!(
              "{}{}{}{}`{url_text}` (in Markdown)  or ``{url_text}`` (in reStructuredText)",
              "this will be rendered as a clickable link, which may not be what you want;",
              " check if the `linkify` plugin is enabled in `myst_enable_extensions`,",
              " which automatically converts URL-like text to links;",
              " alternatively, wrap this in backticks: ",
            )),
          });
          false
        } else if may_not_be_a_permalink(link) {
          diagnostics.push(LinkDiagnostic::NotPermalink {
            uri: link.clone(),
            src: Default::default(),
            sites: Default::default(),
            help: Default::default(),
          });
          true
        } else {
          true
        }
      })
      .cloned()
      .map(|link| {
        let client = client.clone();
        let concurrency = concurrency.clone();
        let redirects = redirects.clone();

        async move {
          let _permit = concurrency.acquire().await.unwrap();

          log::info!("checking {link}");

          let spinner = {
            let msg = format!("still checking {link} ...");
            tokio::spawn(async move {
              loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                log::info!("{}", msg);
              }
            })
          };

          let response = client
            .get(link.clone())
            .header("user-agent", USER_AGENT)
            .timeout(Duration::from_secs(15))
            .send()
            .await;

          spinner.abort();

          match response {
            Ok(response) => {
              let status = response.status();
              if status.is_success()
                || matches!(status, StatusCode::FOUND | StatusCode::NOT_MODIFIED)
              {
                if !redirects.lock().unwrap().contains(response.url()) {
                  Option::<LinkDiagnostic>::None
                } else {
                  let location = response.url();
                  Some(LinkDiagnostic::PermanentlyRedirected {
                    location: location.clone(),
                    uri: link.clone(),
                    src: Default::default(),
                    sites: Default::default(),
                    help: Some(format!(
                      "old: {}\nnew: {}",
                      Yellow.paint(link.to_string()),
                      Green.paint(location.to_string())
                    )),
                  })
                }
              } else if status.is_server_error() {
                Some(LinkDiagnostic::CouldNotCheck {
                  error: anyhow::anyhow!("received server error {status}"),
                  uri: link,
                  src: Default::default(),
                  sites: Default::default(),
                  help: Some("this may be a temporary issue with the server".to_string()),
                })
              } else if status.is_client_error() {
                Some(LinkDiagnostic::NotFound {
                  status,
                  uri: link,
                  src: Default::default(),
                  sites: Default::default(),
                  help: if matches!(status, StatusCode::NOT_FOUND) {
                    Some("this link is likely broken".to_string())
                  } else if matches!(status, StatusCode::FORBIDDEN) {
                    Some("your network may be blocked from accessing this resource".to_string())
                  } else if matches!(status, StatusCode::UNAUTHORIZED) {
                    Some(format!(
                      "{}{}",
                      "this link may require authentication",
                      "; consider changing to a publicly-available resource"
                    ))
                  } else {
                    None
                  },
                })
              } else {
                Some(LinkDiagnostic::CouldNotCheck {
                  error: anyhow::anyhow!("received unexpected status {status}"),
                  uri: link,
                  src: Default::default(),
                  sites: Default::default(),
                  help: Default::default(),
                })
              }
            }
            Err(err) => {
              if err.status().is_some_and(|s| s.is_success()) {
                log::debug!("request returned Err but has a successful status: {err}");
                None
              } else {
                Some(LinkDiagnostic::CouldNotCheck {
                  error: if err.is_timeout() {
                    anyhow::anyhow!("request timed out")
                  } else if err.is_connect() {
                    anyhow::anyhow!("could not connect to server")
                  } else {
                    anyhow::anyhow!("request failed")
                  },
                  uri: link,
                  src: Default::default(),
                  sites: Default::default(),
                  help: if err.is_timeout() || err.is_connect() {
                    Some("this link may not be accessible for some users".to_string())
                  } else {
                    None
                  },
                })
              }
            }
          }
        }
      }),
  );

  while let Some(result) = requests.join_next().await {
    let result = result?;
    if let Some(diagnostic) = result {
      diagnostics.push(diagnostic);
    }
  }

  Ok(diagnostics)
}

static USER_AGENT: &str = r#"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36"#;

static RE_GITHUB_LINK: Lazy<Regex> = Lazy::new(|| {
  Regex::new(
    r#"https?://.*\.?github\.com/[0-9A-Za-z_-]+/[0-9A-Za-z_-]+/(tree|blob)/(master|main)/.+"#,
  )
  .unwrap()
});

static RE_GITHUBUSERCONTENT: Lazy<Regex> = Lazy::new(|| {
  Regex::new(
    r#"https?://raw\.githubusercontent\.com/[0-9A-Za-z_-]+/[0-9A-Za-z_-]+/(master|main)/.+"#,
  )
  .unwrap()
});

fn may_not_be_a_permalink(u: &url::Url) -> bool {
  RE_GITHUB_LINK.is_match(u.as_str()) || RE_GITHUBUSERCONTENT.is_match(u.as_str())
}

fn maybe_special(u: &url::Url) -> Option<String> {
  let host = u.host()?;

  match host {
    url::Host::Domain(domain) => {
      if matches!(domain.split('.').last(), Some("localhost" | "local")) {
        Some(domain.to_string())
      } else {
        None
      }
    }
    url::Host::Ipv4(ip) => {
      if ip.is_loopback() || ip.is_private() || ip.is_unspecified() || ip.is_documentation() {
        Some(ip.to_string())
      } else {
        None
      }
    }
    url::Host::Ipv6(ip) => {
      if ip.is_loopback() || ip.is_unspecified() {
        Some(ip.to_string())
      } else {
        None
      }
    }
  }
}
