use std::{collections::HashMap, fmt::Write as _, io::Write as _, time::Duration};

use ansi_term::Color::{Blue, Red, Yellow};
use deno_lite::DenoLite;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, ThemeCharacters, ThemeStyles};
use swc_core::{
  common::{sync::Lrc, FileName, SourceMap},
  ecma::ast::Expr,
};
use url::Url;

use super::{
  diagnostics::{RelativeSpan, RelativeSpanSet},
  linkcheck::{check_links, collect_links},
  Abort, Bundler,
};

pub struct Linter {
  deno: DenoLite,
  sourcemap: Lrc<SourceMap>,
  trees: Vec<(FileName, Expr)>,
  reporter: GraphicalReportHandler,
}

struct StdErr(std::io::Stderr);

impl std::fmt::Write for StdErr {
  fn write_str(&mut self, s: &str) -> std::fmt::Result {
    self
      .0
      .write(s.as_bytes())
      .map(|_| ())
      .map_err(|_| std::fmt::Error)
  }
}

impl Linter {
  pub fn new(bundler: &Bundler, trees: Vec<(FileName, Expr)>) -> Self {
    Self {
      deno: bundler.deno.clone(),
      sourcemap: bundler.sourcemap.clone(),
      trees,
      reporter: GraphicalReportHandler::new_themed(GraphicalTheme {
        characters: ThemeCharacters {
          advice: "ⓘ ".into(),
          warning: "⚠ ".into(),
          error: "✖ ".into(),
          ..ThemeCharacters::unicode()
        },
        styles: ThemeStyles::ansi(),
      })
      .with_width(120)
      .with_break_words(false),
    }
  }

  pub fn lint(self, abort: &Abort<'_>) -> anyhow::Result<Self> {
    self.linkcheck(abort)?;
    Ok(self)
  }
}

impl Linter {
  fn linkcheck(&self, abort: &Abort<'_>) -> anyhow::Result<()> {
    let links: HashMap<Url, HashMap<FileName, RelativeSpanSet>> = {
      let mut links = vec![];
      self
        .trees
        .iter()
        .for_each(|(_, tree)| collect_links(tree, &mut links));
      links
    }
    .into_iter()
    .filter_map(|(link, span)| {
      RelativeSpan::try_from((&*self.sourcemap, span))
        .map(|span| (link, span))
        .ok()
    })
    .fold(Default::default(), |mut buckets, (link, span)| {
      buckets
        .entry(link)
        .or_default()
        .entry(span.file().name.clone())
        .or_default()
        .push(span);
      buckets
    });

    let task = self
      .deno
      .driver()
      .spawn(check_links(links.keys().cloned().collect()));

    loop {
      self
        .deno
        .driver()
        .block_on(async { tokio::time::sleep(Duration::from_secs(1)).await });

      abort.check()?;

      if task.is_finished() {
        break;
      }
    }

    let mut diagnostics = self.deno.driver().block_on(task)??;

    sort_diagnostics_by_severity(&mut diagnostics);

    let mut stderr = StdErr(std::io::stderr());

    diagnostics.iter_mut().try_for_each(|diag| {
      let files = links.get(diag.url()).expect("should be in the map");
      for spans in files.values() {
        self.reporter.render_report(
          &mut stderr,
          diag
            .with_labels(diag.to_labels(spans.0.iter()))
            .with_source(spans.0.first()),
        )?;
        stderr.write_str("\n\n")?;
      }
      anyhow::Result::<()>::Ok(())
    })?;

    let stats = diagnostics
      .iter()
      .fold((0, 0, 0, 0), |(count, errors, warnings, infos), d| {
        let severity = d.severity();
        let count = count + 1;
        let errors = errors + matches!(severity, Some(miette::Severity::Error) | None) as usize;
        let warnings = warnings + matches!(severity, Some(miette::Severity::Warning)) as usize;
        let infos = infos + matches!(severity, Some(miette::Severity::Advice)) as usize;
        (count, errors, warnings, infos)
      });

    log::info!(
      "checked {} links; {} errors, {} warnings, {} infos",
      stats.0,
      if stats.1 > 0 {
        Red.paint(stats.1.to_string())
      } else {
        stats.1.to_string().into()
      },
      if stats.2 > 0 {
        Yellow.paint(stats.2.to_string())
      } else {
        stats.2.to_string().into()
      },
      if stats.3 > 0 {
        Blue.paint(stats.3.to_string())
      } else {
        stats.3.to_string().into()
      }
    );

    Ok(())
  }
}

fn sort_diagnostics_by_severity<D: miette::Diagnostic>(diagnostics: &mut [D]) {
  diagnostics.sort_by_key(|a| (a.severity(), a.code().map(|c| c.to_string())))
}
