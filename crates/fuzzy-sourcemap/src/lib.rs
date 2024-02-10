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

pub trait SourceLoader {
  fn load_source(&self, current_file: &FileName, last_file: &Option<FileName>) -> Option<String>;
}

pub struct FuzzySourceMap {
  sources: Lrc<SourceMap>,
  loader: Option<Box<dyn SourceLoader>>,

  this_file: Option<FileName>,
  this_para: Option<Span>,
  this_span: Option<Span>,
}

impl FuzzySourceMap {
  pub fn next_span(
    &mut self,
    file_name: Option<FileName>,
    line_number: Option<usize>,
    snippet: Option<&str>,
  ) -> Option<Span> {
    match file_name {
      Some(name) => {
        self.this_file = Some(name);
      }
      None => {}
    };

    let Some(ref file_name) = self.this_file else {
      self.this_para = None;
      self.this_span = None;
      return None;
    };

    let source = self.sources.get_source_file(file_name);

    let source = match source {
      Some(source) => Some(source),
      None => match self.loader {
        Some(ref loader) => {
          let source = loader.load_source(file_name, &self.this_file);
          match source {
            Some(source) => Some(self.sources.new_source_file(file_name.clone(), source)),
            None => None,
          }
        }
        None => None,
      },
    };

    let Some(source) = source else {
      self.this_para = None;
      self.this_span = None;
      return self.this_span;
    };

    let snippet = match snippet {
      Some(s) if s == "" => None,
      _ => snippet,
    };

    let para = match line_number {
      Some(line_number) => find_nearest_paragraph(&*source, line_number).or(self.this_para),
      None => match snippet {
        Some(_) => self.this_para,
        None => None,
      },
    };

    let Some(para) = para else {
      self.this_span = None;
      return None;
    };

    let span = match snippet {
      None => Some(para),
      Some(snippet) => {
        let span = find_span_with_snippet(&self.sources, &source, self.this_span, para, snippet);
        span.or(Some(para))
      }
    };

    let para = match span {
      Some(span) if span != para => {
        if let Some(actual_line) = source.lookup_line(span.lo()) {
          find_nearest_paragraph(&source, actual_line + 1).unwrap_or(para)
        } else {
          para
        }
      }
      _ => para,
    };

    self.this_para = Some(para);
    self.this_span = span;

    self.this_span
  }
}

impl FuzzySourceMap {
  pub fn new(sources: Lrc<SourceMap>, loader: Option<Box<dyn SourceLoader>>) -> FuzzySourceMap {
    FuzzySourceMap {
      sources,
      loader,
      this_file: None,
      this_para: None,
      this_span: None,
    }
  }
}

fn find_span_with_snippet(
  sources: &SourceMap,
  file: &SourceFile,
  prev_span: Option<Span>,
  paragraph: Span,
  snippet: &str,
) -> Option<Span> {
  let find_in_region = |region| {
    sources
      .with_snippet_of_span(region, |text| {
        let start = text.find(snippet);
        match start {
          None => None,
          Some(start) => Some(region.from_inner_byte_pos(start, start + snippet.len())),
        }
      })
      .ok()
      .unwrap_or(None)
  };

  let sub_paragraph = prev_span.and_then(|span| {
    Some((
      Span::from((span.hi(), paragraph.hi())),
      span,
      Span::from((paragraph.lo(), span.lo())),
    ))
  });

  let processed_text = Span::from((file.start_pos, paragraph.lo()));
  let remaining_text = Span::from((paragraph.hi(), file.end_pos));

  let found = sub_paragraph
    .and_then(|(tail, previous_span, head)| {
      find_in_region(tail)
        .or_else(|| find_in_region(previous_span))
        .or_else(|| find_in_region(head))
    })
    .or_else(|| find_in_region(paragraph))
    .or_else(|| find_in_region(remaining_text))
    .or_else(|| find_in_region(processed_text));

  if let Some(found) = found {
    return Some(found);
  }

  let expected_lines = snippet.lines().collect::<Vec<_>>();

  let this_line = file
    .lookup_line(paragraph.lo())
    .expect("para.lo() should be within this file");

  let last_line = file.count_lines();
  let last_line = last_line - expected_lines.len();

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
      let lo = file.line_bounds(line_no).0 + BytePos(indent as u32);
      let hi = file.line_bounds(line_no + expected_lines.len() - 1).1;
      Some(Span::from((lo, hi)))
    }
  })
}

fn find_nearest_paragraph(src: &SourceFile, line_number: usize) -> Option<Span> {
  let line_number = one_indexed!(line_number);

  let nearest_contentful_line = {
    let mut current_line = line_number;
    loop {
      let Some(text) = src.get_line(current_line) else {
        break;
      };
      if !is_empty_or_whitespace(&*text) {
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
      if is_empty_or_whitespace(&*next_line_text) {
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
      let indent = indentation_of(&*text);
      if indent == "" {
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
      if is_empty_or_whitespace(&*next_line_text) {
        break;
      } else {
        current_line = next_line;
      }
    }
    current_line
  };

  if nearest_contentful_line >= src.lines.len() || nearest_top_level_line >= src.lines.len() {
    return None;
  }

  let (lower, _) = src.line_bounds(nearest_top_level_line);
  let (_, upper) = src.line_bounds(nearest_contentful_line);

  Some((lower, upper).into())
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
mod tests {
  use std::path::PathBuf;

  use glob::glob;
  use miette::{
    Diagnostic, GraphicalReportHandler, LabeledSpan, NamedSource, Severity, SourceCode,
  };
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

  #[derive(Serialize, Deserialize, Debug, Diagnostic, Error)]
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
    Found((SourceCodeQuery, Span)),
  }

  #[derive(Debug)]
  struct SourceCodeReport {
    source: NamedSource,
    span: Vec<LabeledSpan>,
    help: Box<String>,
  }

  impl std::fmt::Display for SourceCodeReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "found {}", self.span[0].label().unwrap())
    }
  }

  impl std::error::Error for SourceCodeReport {}

  impl Diagnostic for SourceCodeReport {
    fn severity(&self) -> Option<Severity> {
      Some(Severity::Advice)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
      Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
      Some(Box::new(self.span.iter().cloned()))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
      Some(Box::new(&self.help))
    }
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
    FileName::Custom(name.to_string())
  }

  fn print_report(results: &Vec<SourceCodeResult>) {
    let mut output = String::new();
    let handler = GraphicalReportHandler::new();
    for result in results {
      match result {
        SourceCodeResult::NotFound(query) => handler.render_report(&mut output, query).unwrap(),
        SourceCodeResult::Found((query, span)) => {
          let fixture = get_fixture();

          let file_name = fixture.span_to_filename(*span);
          let file = fixture.get_source_file(&file_name).unwrap();

          let source = NamedSource::new(file_name.to_string(), file.src.clone());

          let span = vec![LabeledSpan::new(
            Some(query.node_name.clone()),
            (span.lo().0 - file.start_pos.0) as usize,
            (span.hi().0 - span.lo().0) as usize,
          )];

          let help = {
            let line = match query.line_number {
              Some(line) => format!("line {}", line),
              None => "line ?".to_string(),
            };
            match query.raw_source {
              Some(ref source) if source.len() > 0 => {
                Box::new(format!("{}, raw source:\n\n{}", line, source))
              }
              _ => Box::new(format!("{}, raw source: ?", line)),
            }
          };

          let report = SourceCodeReport { source, span, help };

          handler.render_report(&mut output, &report).unwrap();
        }
      };
      output.push('\n');
    }
    println!("{}", output);
  }

  fn test_doc(query_path: PathBuf) -> anyhow::Result<()> {
    let queries: Vec<SourceCodeQuery> =
      serde_yaml::from_str(std::fs::read_to_string(query_path.clone())?.as_str())?;

    let doc_path = query_path.clone().with_extension("");

    let mut mapper = FuzzySourceMap::new(get_fixture(), None);

    let mut results: Vec<SourceCodeResult> = vec![];

    for query in queries {
      let span = mapper.next_span(
        query
          .file_name
          .clone()
          .and_then(|f| Some(fmt_fixture_name(f.as_str()))),
        if query.line_number == Some(0) {
          None
        } else {
          query.line_number
        },
        query.raw_source.as_deref(),
      );
      match span {
        Some(span) => {
          results.push(SourceCodeResult::Found((query, span)));
        }
        None => {
          results.push(SourceCodeResult::NotFound(query));
        }
      };
    }

    std::panic::catch_unwind(|| print_report(&results)).unwrap();

    insta::assert_yaml_snapshot!(get_fixture_name(&doc_path).to_string(), &results);

    if std::env::var("PRINT_REPORT").is_ok() {
      print_report(&results);
    }

    Ok(())
  }

  #[fixture("tests/fixtures/*.*.yaml")]
  fn test_sphinx_docs(queries: PathBuf) {
    test_doc(queries).unwrap();
  }
}
