use swc_core::common::{sync::Lrc, FileName, SourceMap, Span};

pub trait SourceLoader {
  fn load_source(&mut self, current_file: FileName, last_file: Option<FileName>) -> Option<&str>;
}

pub struct SourceFinder {
  sources: Lrc<SourceMap>,
  loader: Box<dyn SourceLoader>,
  current_file: Option<FileName>,
  current_span: Option<Span>,
}

impl SourceFinder {
  pub fn next_span(
    &mut self,
    current_file: FileName,
    line_number: Option<usize>,
    raw_source: Option<&str>,
  ) -> Option<Span> {
    todo!()
  }
}
