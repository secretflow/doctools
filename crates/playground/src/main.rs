use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
#[error("invalid link")]
#[diagnostic(code(E0001), help("try adding https://"))]
struct InvalidLinkError {
  #[source_code]
  src: NamedSource,
  #[label("link defined here")]
  definition: SourceSpan,
  #[label("link used here")]
  use_site: SourceSpan,
}

#[derive(Debug, Error)]
#[error("invalid link 2")]
struct InvalidLinkError2;

impl Diagnostic for InvalidLinkError2 {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    None
  }

  fn severity(&self) -> Option<miette::Severity> {
    None
  }

  fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    None
  }

  fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    None
  }

  fn source_code(&self) -> Option<&dyn miette::SourceCode> {
    None
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
    None
  }

  fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
    None
  }

  fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
    None
  }
}

fn main() {}
