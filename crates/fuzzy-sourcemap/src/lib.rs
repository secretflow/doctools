use swc_core::common::{sync::Lrc, FileName, SourceFile, SourceMap, Span};

pub mod paragraph;
mod whitespace;

#[macro_export]
macro_rules! one_indexed {
  ($x:expr) => {{
    if $x <= 0 {
      panic!("Index must be greater than 0")
    } else {
      $x - 1
    }
  }};
}

pub trait SourceLoader {
  fn load_source(
    &mut self,
    current_file: &FileName,
    last_file: &Option<FileName>,
  ) -> Option<String>;
}

pub struct SourceFinder {
  sources: Lrc<SourceMap>,
  loader: Box<dyn SourceLoader>,

  this_file: Option<FileName>,
  this_para: Option<Span>,
  this_span: Option<Span>,
}

impl SourceFinder {
  pub fn next_span(
    &mut self,
    file_name: FileName,
    line_number: Option<usize>,
    snippet: Option<&str>,
  ) -> Option<Span> {
    todo!()
  }
}
