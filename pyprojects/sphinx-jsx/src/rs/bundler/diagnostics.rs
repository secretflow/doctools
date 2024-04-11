use std::{
  cmp::Ordering,
  hash::{Hash, Hasher},
};

use swc_core::common::{source_map::Pos as _, sync::Lrc, BytePos, SourceFile, SourceMap, Span};

pub type DiagnosticSource = miette::NamedSource<Lrc<String>>;

#[derive(Debug, Default)]
pub struct DeferredSource(Option<DiagnosticSource>);

impl DeferredSource {
  pub fn set(&mut self, source: Option<DiagnosticSource>) {
    self.0 = source
  }

  pub fn set_default<F: Fn() -> DiagnosticSource>(&mut self, new: F) {
    match self.0 {
      None => self.0 = Some(new()),
      Some(_) => {}
    }
  }
}

impl miette::SourceCode for DeferredSource {
  fn read_span<'a>(
    &'a self,
    span: &miette::SourceSpan,
    context_lines_before: usize,
    context_lines_after: usize,
  ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
    match self {
      DeferredSource(Some(source)) => {
        source.read_span(span, context_lines_before, context_lines_after)
      }
      DeferredSource(None) => Err(miette::MietteError::OutOfBounds),
    }
  }
}

#[derive(Debug, Clone)]
pub struct RelativeSpan {
  file: Lrc<SourceFile>,
  span: miette::SourceSpan,
}

impl PartialEq for RelativeSpan {
  fn eq(&self, other: &Self) -> bool {
    self.file.name == other.file.name && self.span == other.span
  }
}

impl Eq for RelativeSpan {}

impl Hash for RelativeSpan {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.file.name.hash(state);
    self.span.hash(state);
  }
}

impl From<RelativeSpan> for miette::SourceSpan {
  fn from(value: RelativeSpan) -> Self {
    value.span
  }
}

impl TryFrom<(&SourceMap, Span)> for RelativeSpan {
  type Error = anyhow::Error;

  fn try_from((sourcemap, span): (&SourceMap, Span)) -> anyhow::Result<Self> {
    let file_idx = sourcemap
      .files()
      .binary_search_by(
        |file| match (file.start_pos.cmp(&span.lo), file.end_pos.cmp(&span.hi)) {
          (Ordering::Less | Ordering::Equal, Ordering::Greater | Ordering::Equal) => {
            Ordering::Equal
          }
          (Ordering::Less, _) => Ordering::Less,
          _ => Ordering::Greater,
        },
      )
      .map_err(|_| anyhow::anyhow!("span {span:?} is not from any source file"))?;
    let file = sourcemap.files()[file_idx].clone();
    let offset = span.lo - file.start_pos;
    let length = span.hi - span.lo;
    let span = miette::SourceSpan::new(offset.to_usize().into(), length.to_usize());
    Ok(Self { file, span })
  }
}

impl RelativeSpan {
  pub fn file(&self) -> &SourceFile {
    &self.file
  }

  pub fn span(&self) -> miette::SourceSpan {
    self.span
  }

  pub fn labeled(&self, label: &str) -> miette::LabeledSpan {
    miette::LabeledSpan::new_with_span(Some(label.into()), self.span)
  }

  pub fn source(&self) -> DiagnosticSource {
    DiagnosticSource::new(self.file.name.to_string(), self.file.src.clone())
  }

  pub fn text(&self) -> &str {
    &self.file.src[self.span.offset()..self.span.offset() + self.span.len()]
  }
}

#[derive(Debug, Default, Clone)]
pub struct RelativeSpanSet(pub Vec<RelativeSpan>);

impl RelativeSpanSet {
  pub fn push(&mut self, span: RelativeSpan) {
    let found = self.0.binary_search_by(|test| {
      match (
        test.span.offset().cmp(&span.span.offset()),
        test.span.len().cmp(&span.span.len()),
      ) {
        (Ordering::Less | Ordering::Equal, Ordering::Greater | Ordering::Equal) => Ordering::Equal,
        (Ordering::Less, _) => Ordering::Less,
        _ => Ordering::Greater,
      }
    });
    match found {
      Ok(_) => {}
      Err(idx) => self.0.insert(idx, span),
    }
  }
}

#[derive(Clone)]
pub struct RelativeOffset {
  file: Lrc<SourceFile>,
  offset: miette::SourceOffset,
}

impl TryFrom<(&SourceMap, BytePos)> for RelativeOffset {
  type Error = anyhow::Error;

  fn try_from((sourcemap, pos): (&SourceMap, BytePos)) -> Result<Self, Self::Error> {
    let RelativeSpan { file, span } = RelativeSpan::try_from((sourcemap, (pos, pos).into()))?;
    Ok(Self {
      file,
      offset: span.offset().into(),
    })
  }
}

impl RelativeOffset {
  pub fn file(&self) -> &SourceFile {
    &self.file
  }

  pub fn offset(&self) -> usize {
    self.offset.offset()
  }
}

pub fn ensure_char_boundary(sourcemap: &SourceMap, pos: BytePos) -> anyhow::Result<BytePos> {
  let offset = RelativeOffset::try_from((sourcemap, pos))?;

  let src = &*offset.file().src;
  let offset = offset.offset();

  if src.is_char_boundary(offset) {
    Ok(pos)
  } else if offset > src.len() {
    let difference = offset - src.len();
    Ok(pos - BytePos::from_usize(difference))
  } else {
    let mut difference = 0;
    while !src.is_char_boundary(offset + difference) {
      difference += 1;
    }
    Ok(pos + BytePos::from_usize(difference))
  }
}
