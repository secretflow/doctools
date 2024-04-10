use std::path::{Path, PathBuf};

use swc_core::common::{sync::Lrc, BytePos, FileName, SourceFile, SourceMap, Span};

#[macro_export]
macro_rules! one_indexed {
  ($x:expr) => {{
    if $x <= 0 {
      panic!("index must be greater than 0")
    } else {
      $x - 1
    }
  }};
}

type SomePathResolver = Box<dyn PathResolver + Send + Sync>;

pub trait PathResolver {
  fn resolve(&self, path: Option<&str>, base: Option<&Path>) -> Option<PathBuf>;
}

impl Default for SomePathResolver {
  fn default() -> Self {
    Box::new(DefaultResolver)
  }
}

struct DefaultResolver;

impl PathResolver for DefaultResolver {
  fn resolve(&self, path: Option<&str>, base: Option<&Path>) -> Option<PathBuf> {
    match path {
      Some(path) => Some(PathBuf::from(path)),
      None => base.map(PathBuf::from),
    }
  }
}

pub struct FuzzySourceMap {
  sources: Lrc<SourceMap>,
  resolver: SomePathResolver,
  current_file: Option<FileName>,
  last_region: Option<Span>,
  last_span: Option<Span>,
}

enum FileChange {
  NotChanged,
  Changed,
}

enum SpanSearch<'a> {
  FullyQualified {
    region: Span,
    last_span: Option<Span>,
    text: &'a str,
  },
  TextOnly {
    last_region: Span,
    last_span: Option<Span>,
    text: &'a str,
  },
  RegionOnly {
    region: Span,
  },
  None,
}

enum SpanResult {
  HighConfidence {
    region: Span,
    span: Span,
  },
  LowConfidence {
    span: Span,
    #[allow(dead_code)]
    reason: LowConfidence,
  },
  RegionOnly {
    region: Span,
  },
  NotFound,
}

#[derive(Debug, thiserror::Error)]
enum LowConfidence {
  #[error("searching backwards")]
  SearchingBackwards,
  #[error("search text too short")]
  SearchTextTooShort,
  #[error("found beyond requested lines")]
  SearchBeyondRegion,
}

impl FuzzySourceMap {
  pub fn feed(
    &mut self,
    file_name: Option<&str>,
    line: Option<usize>,
    text: Option<&str>,
  ) -> Option<Span> {
    let (file, file_change, search) = self.preconditions(file_name, line, text)?;

    self.current_file = Some(file.name.clone());

    if matches!(file_change, FileChange::Changed) {
      self.last_region = None;
      self.last_span = None;
    }

    let result = self.query(file.clone(), search);

    #[cfg(test)]
    {
      if log::log_enabled!(log::Level::Trace) {
        let (confidence, span, region) = match result {
          SpanResult::HighConfidence { span, region } => {
            ("high confidence".to_string(), Some(span), Some(region))
          }
          SpanResult::LowConfidence { span, ref reason } => {
            (format!("low confidence: {}", reason), Some(span), None)
          }
          SpanResult::RegionOnly { .. } => ("".to_string(), None, None),
          SpanResult::NotFound => ("".to_string(), None, None),
        };
        if let Some(span) = span {
          use self::debug_tests::{to_labeled_span, to_named_source};
          let mut labels = vec![
            to_labeled_span(&format!("{:?}", span), &file, span),
            to_labeled_span(
              &self
                .sources
                .with_snippet_of_span(span, |found| format!("{:?}", found))
                .unwrap(),
              &file,
              span,
            ),
          ];
          if let Some(region) = region {
            labels.push(to_labeled_span("in this paragraph", &file, region))
          }
          let report = miette::miette!(
            severity = miette::Severity::Advice,
            labels = labels,
            "final span ({confidence})"
          )
          .with_source_code(to_named_source(&file));
          log::trace!("\n{:?}", report);
        } else {
          log::trace!("final span: None");
        }
      }
    }

    match result {
      SpanResult::HighConfidence { region, span } => {
        self.last_region = Some(region);
        self.last_span = Some(span);
        Some(span)
      }
      SpanResult::LowConfidence { span, .. } => Some(span),
      SpanResult::RegionOnly { region } => {
        self.last_region = Some(region);
        Some(region)
      }
      SpanResult::NotFound => None,
    }
  }

  fn preconditions<'a>(
    &self,
    file_name: Option<&str>,
    line: Option<usize>,
    text: Option<&'a str>,
  ) -> Option<(Lrc<SourceFile>, FileChange, SpanSearch<'a>)> {
    let Some(file_name) = self.resolver.resolve(
      file_name,
      match self.current_file {
        Some(FileName::Real(ref path)) => Some(path),
        _ => None,
      },
    ) else {
      log::trace!("cannot resolve {file_name:?}");
      return None;
    };

    let file = match self
      .sources
      .get_source_file(&FileName::Real(file_name.clone()))
    {
      Some(file) => Some(file),
      None => self.sources.load_file(&file_name).ok(),
    };

    let file = file?;

    let (file_change, last_region, last_span) = if Some(&file.name) == self.current_file.as_ref() {
      (FileChange::NotChanged, self.last_region, self.last_span)
    } else {
      log::trace!("file changed: {:?} -> {:?}", self.current_file, &file.name);
      (FileChange::Changed, None, None)
    };

    let region = line.and_then(|line| find_nearest_paragraph(&file, line));

    let text = text.map(str::trim_end).unwrap_or_default();

    let search = if let Some(region) = region {
      if !text.is_empty() {
        SpanSearch::FullyQualified {
          region,
          last_span,
          text,
        }
      } else {
        SpanSearch::RegionOnly { region }
      }
    } else if text.len() > 1 {
      let last_region = last_region?;
      SpanSearch::TextOnly {
        last_region,
        last_span,
        text,
      }
    } else {
      log::trace!("will not proceed when neither line nor text is provided, or text is too short");
      SpanSearch::None
    };

    Some((file, file_change, search))
  }

  fn query(&self, file: Lrc<SourceFile>, query: SpanSearch<'_>) -> SpanResult {
    match query {
      SpanSearch::FullyQualified {
        region,
        last_span,
        text,
      } => {
        if let Some(span) = find_span_with_exact_text(&self.sources, &file, last_span, region, text)
          .or_else(|| find_span_with_suffix_match(&file, region, text))
        {
          let region = file
            .lookup_line(span.lo)
            .and_then(|line| find_nearest_paragraph(&file, line + 1))
            .unwrap_or(region);
          if text.len() > 1 {
            if let Some(last_span) = last_span {
              if span.lo >= last_span.lo {
                SpanResult::HighConfidence { region, span }
              } else {
                SpanResult::LowConfidence {
                  span,
                  reason: LowConfidence::SearchingBackwards,
                }
              }
            } else {
              SpanResult::HighConfidence { region, span }
            }
          } else {
            SpanResult::LowConfidence {
              span,
              reason: LowConfidence::SearchTextTooShort,
            }
          }
        } else {
          SpanResult::RegionOnly { region }
        }
      }
      SpanSearch::TextOnly {
        last_region,
        last_span,
        text,
      } => {
        if let Some(span) =
          find_span_with_exact_text(&self.sources, &file, last_span, last_region, text)
            .or_else(|| find_span_with_suffix_match(&file, last_region, text))
        {
          let region = file
            .lookup_line(span.lo)
            .and_then(|line| find_nearest_paragraph(&file, line + 1))
            .unwrap_or(last_region);
          if region.source_equal(last_region) {
            if text.len() == 1 {
              SpanResult::LowConfidence {
                span,
                reason: LowConfidence::SearchTextTooShort,
              }
            } else if span.lo < last_region.lo {
              SpanResult::LowConfidence {
                span,
                reason: LowConfidence::SearchingBackwards,
              }
            } else {
              SpanResult::HighConfidence { region, span }
            }
          } else {
            SpanResult::LowConfidence {
              span,
              reason: LowConfidence::SearchBeyondRegion,
            }
          }
        } else {
          SpanResult::NotFound
        }
      }
      SpanSearch::RegionOnly { region } => SpanResult::RegionOnly { region },
      SpanSearch::None => SpanResult::NotFound,
    }
  }
}

impl FuzzySourceMap {
  pub fn new(sources: Lrc<SourceMap>, resolver: SomePathResolver) -> FuzzySourceMap {
    FuzzySourceMap {
      sources,
      resolver,
      current_file: None,
      last_region: None,
      last_span: None,
    }
  }
}

fn find_nearest_paragraph(src: &SourceFile, line: usize) -> Option<Span> {
  let line = one_indexed!(line);

  let nearest_contentful_line = {
    let mut current_line = line;
    loop {
      let Some(text) = src.get_line(current_line) else {
        break;
      };
      if !is_empty_or_whitespace(&text) {
        break;
      } else {
        let Some(next_line) = current_line.checked_sub(1) else {
          break;
        };
        current_line = next_line;
      }
    }
    loop {
      let Some(next_line) = current_line.checked_sub(1) else {
        break;
      };
      let Some(next_line_text) = src.get_line(next_line) else {
        break;
      };
      if is_empty_or_whitespace(&next_line_text) {
        break;
      } else {
        current_line = next_line;
      }
    }
    current_line
  };

  let nearest_top_level_line = {
    let mut current_line = nearest_contentful_line;
    loop {
      let Some(text) = src.get_line(current_line) else {
        break;
      };
      let indent = indentation_of(&text);
      if indent.is_empty() {
        break;
      } else {
        let Some(next_line) = current_line.checked_sub(1) else {
          break;
        };
        current_line = next_line;
      }
    }
    loop {
      let Some(next_line) = current_line.checked_sub(1) else {
        break;
      };
      let Some(next_line_text) = src.get_line(next_line) else {
        break;
      };
      if is_empty_or_whitespace(&next_line_text) {
        break;
      } else {
        current_line = next_line;
      }
    }
    current_line
  };

  let nearest_contentful_line = {
    let mut current_line = nearest_contentful_line;
    loop {
      let next_line = current_line + 1;
      let Some(text) = src.get_line(next_line) else {
        break;
      };
      if is_empty_or_whitespace(&text) {
        break;
      } else {
        current_line = next_line;
      }
    }
    current_line
  };

  log::trace!("line_number = {line}, nearest_contentful_line = {nearest_contentful_line}, nearest_top_level_line = {nearest_top_level_line}");

  if nearest_contentful_line >= src.lines.len() || nearest_top_level_line >= src.lines.len() {
    return None;
  }

  let (lower, _) = src.line_bounds(nearest_top_level_line);
  let (_, upper) = src.line_bounds(nearest_contentful_line);

  let (lower, upper) = trim_line_bounds(src, (lower, upper));

  Some((lower, upper).into())
}

fn find_span_with_exact_text(
  sources: &SourceMap,
  file: &SourceFile,
  prev_span: Option<Span>,
  region: Span,
  text: &str,
) -> Option<Span> {
  #[cfg(test)]
  {
    if log::log_enabled!(log::Level::Trace) {
      use self::debug_tests::{to_labeled_span, to_named_source};
      use miette::miette;
      let mut labels = vec![
        to_labeled_span(&format!("paragraph {:?}", region), file, region),
        to_labeled_span(&format!("searching for {:?}", text), file, region),
      ];
      if let Some(last_seen) = prev_span {
        labels.push(to_labeled_span(
          &format!("last seen {:?}", last_seen),
          file,
          last_seen,
        ));
      }
      let report = miette!(
        severity = miette::Severity::Advice,
        labels = labels,
        "search input"
      )
      .with_source_code(to_named_source(file));
      log::trace!("\n{:?}", report);
    }
  }

  let find_in_region = |region: Span, name| {
    if region.lo >= region.hi {
      return None;
    }
    sources
      .with_snippet_of_span(region, |section| {
        #[cfg(test)]
        {
          if log::log_enabled!(log::Level::Trace) {
            use self::debug_tests::{to_labeled_span, to_named_source};
            use miette::miette;
            let report = miette!(
              severity = miette::Severity::Advice,
              labels = vec![to_labeled_span(
                &format!("{} {:?}", name, region),
                file,
                region
              )],
              "{}",
              name
            )
            .with_source_code(to_named_source(file));
            log::trace!("\n{:?}", report);
          }
        }
        let _ = name;
        section
          .find(text)
          .map(|start| region.from_inner_byte_pos(start, start + text.len()))
      })
      .unwrap_or(None)
  };

  let sub_paragraph = prev_span.and_then(|prev_span| {
    if region.contains(prev_span) {
      Some((
        Span::from((prev_span.hi(), region.hi())),
        prev_span,
        Span::from((region.lo(), prev_span.lo())),
      ))
    } else {
      None
    }
  });

  let remaining_text = Span::from((region.lo(), file.end_pos));

  sub_paragraph
    .and_then(|(remaining_paragraph, last_seen, before_last_seen)| {
      find_in_region(last_seen, stringify!(last_seen))
        .or_else(|| find_in_region(remaining_paragraph, stringify!(remaining_paragraph)))
        .or_else(|| find_in_region(before_last_seen, stringify!(before_last_seen)))
    })
    .or_else(|| find_in_region(remaining_text, stringify!(remaining_text)))
}

fn find_span_with_suffix_match(file: &SourceFile, region: Span, text: &str) -> Option<Span> {
  let expected_lines = text.lines().collect::<Vec<_>>();

  let this_line = file
    .lookup_line(region.lo())
    .expect("para.lo() should be within this file");

  let last_line = file.count_lines();
  let last_line = last_line.saturating_sub(expected_lines.len());

  (this_line..=last_line).find_map(|line_no| {
    let head = &*file
      .get_line(line_no)
      .expect("line_no should be within this file");

    let indent = head.ends_with(
      expected_lines
        .first()
        .expect("snippet should have at least one line"),
    );

    if !indent {
      return None;
    }

    let matched = expected_lines.iter().enumerate().all(|(offset, line)| {
      let expected = *line;
      let actual = &*file
        .get_line(line_no + offset)
        .expect("line_no should be within this file");
      actual.ends_with(expected)
    });

    if !matched {
      None
    } else {
      let lower = file.line_bounds(line_no).0;
      let upper = file.line_bounds(line_no + expected_lines.len() - 1).1;
      let (lower, upper) = trim_line_bounds(file, (lower, upper));
      Some(Span::from((lower, upper)))
    }
  })
}

fn trim_line_bounds(src: &SourceFile, (lower, upper): (BytePos, BytePos)) -> (BytePos, BytePos) {
  let upper = {
    let begin = lower.0 - src.start_pos.0;
    let end = upper.0 - src.start_pos.0;
    if src.src[begin as usize..end as usize].ends_with('\n') {
      upper - BytePos(1)
    } else {
      upper
    }
  };
  (lower, upper)
}

fn is_empty_or_whitespace(text: &str) -> bool {
  text.chars().all(|c| c.is_ascii_whitespace())
}

fn indentation_of(text: &str) -> &str {
  for (idx, ch) in text.char_indices() {
    if !matches!(ch, ' ' | '\t') {
      return &text[..idx];
    }
  }
  text
}

#[cfg(test)]
mod debug_tests {
  use miette::{LabeledSpan, NamedSource};
  use swc_core::common::{SourceFile, Span};

  pub fn to_named_source(source: &SourceFile) -> NamedSource {
    NamedSource::new(format!("{}", source.name), source.src.clone())
  }

  pub fn to_labeled_span(label: &str, source: &SourceFile, span: Span) -> LabeledSpan {
    LabeledSpan::new(
      Some(label.into()),
      (span.lo().0 - source.start_pos.0) as usize,
      (span.hi().0 - span.lo().0) as usize,
    )
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use glob::glob;
  use once_cell::sync::Lazy;
  use relative_path::PathExt as _;
  use serde::{Deserialize, Serialize};
  use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap};
  use swc_core::{common::Span, testing::fixture};
  use thiserror::Error;

  use crate::FuzzySourceMap;

  static FIXTURE_ROOT: Lazy<String> =
    Lazy::new(|| format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR")));

  static SOURCES: Lazy<Vec<(FileName, String)>> = Lazy::new(load_fixture);

  #[derive(Serialize, Deserialize, Debug, Error)]
  #[error(
    "source code not found for {node_name}:
  file_name: {file_name:?}
  line_number: {line_number:?}
  raw_source: {raw_source:?}"
  )]
  struct SourceCodeQuery {
    file_name: Option<String>,
    line_number: Option<usize>,
    raw_source: Option<String>,
    node_name: String,
  }

  #[derive(Debug, Serialize, Deserialize)]
  enum SourceCodeResult {
    NotFound(SourceCodeQuery),
    Found {
      query: SourceCodeQuery,
      span: Span,
      slice: String,
    },
  }

  fn load_fixture() -> Vec<(FileName, String)> {
    let mut sources = vec![];
    for entry in glob(format!("{}/**/*.*.yaml", *FIXTURE_ROOT).as_str()).unwrap() {
      let entry = match entry {
        Ok(entry) => entry,
        Err(_) => continue,
      };
      if !entry.is_file() {
        continue;
      }
      let doc_path = entry.clone().with_extension("");
      let content = std::fs::read_to_string(doc_path.clone()).unwrap();
      sources.push((get_fixture_name(&doc_path), content));
    }
    sources
  }

  fn get_fixture() -> Lrc<SourceMap> {
    let sources: Lrc<SourceMap> = Default::default();
    for (file_name, content) in SOURCES.iter() {
      sources.new_source_file(file_name.clone(), content.clone());
    }
    sources
  }

  #[allow(dead_code)]
  fn get_fixture_file_from_name(sources: &Lrc<SourceMap>, file_name: &str) -> Lrc<SourceFile> {
    sources
      .get_source_file(&fmt_fixture_name(file_name))
      .unwrap()
  }

  #[allow(dead_code)]
  fn get_fixture_file_from_path(sources: &Lrc<SourceMap>, file_name: &PathBuf) -> Lrc<SourceFile> {
    sources
      .get_source_file(&get_fixture_name(file_name))
      .unwrap()
  }

  fn get_fixture_name(path: &PathBuf) -> FileName {
    let name = path.relative_to(&*FIXTURE_ROOT.clone()).unwrap();
    fmt_fixture_name(name.to_string().as_str())
  }

  fn fmt_fixture_name(name: &str) -> FileName {
    FileName::Real(PathBuf::from(name))
  }

  fn test_doc(query_path: PathBuf) -> anyhow::Result<()> {
    let queries: Vec<SourceCodeQuery> =
      serde_yaml::from_str(std::fs::read_to_string(query_path.clone())?.as_str())?;

    let doc_path = query_path.clone().with_extension("");

    let mut mapper = FuzzySourceMap::new(get_fixture(), Default::default());

    let mut results: Vec<SourceCodeResult> = vec![];

    for query in queries {
      let span = mapper.feed(
        query.file_name.as_deref(),
        if query.line_number == Some(0) {
          None
        } else {
          query.line_number
        },
        query.raw_source.as_deref(),
      );
      match span {
        Some(span) => get_fixture()
          .with_snippet_of_span(span, |slice| {
            results.push(SourceCodeResult::Found {
              query,
              span,
              slice: slice.into(),
            });
          })
          .unwrap(),
        None => {
          results.push(SourceCodeResult::NotFound(query));
        }
      };
    }

    insta::assert_yaml_snapshot!(get_fixture_name(&doc_path).to_string(), &results);

    Ok(())
  }

  #[fixture("tests/fixtures/*.*.yaml")]
  fn test_sphinx_docs(queries: PathBuf) {
    let _ = env_logger::builder()
      .format_indent(None)
      .is_test(true)
      .try_init();
    test_doc(queries).unwrap();
  }
}
