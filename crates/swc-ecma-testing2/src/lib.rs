use std::{fmt::Debug, path::PathBuf};

use serde::de::DeserializeOwned;
use swc_core::{
  common::{comments::Comments, sync::Lrc, EqIgnoreSpan, FileName, SourceFile, SourceMap},
  ecma::{
    self,
    ast::{EsVersion, Expr, Module},
    codegen::{text_writer::JsWriter, Config, Emitter, Node},
    parser::{parse_file_as_expr, Syntax},
    visit::{Fold, FoldWith as _},
  },
};
use swc_error_reporters::handler::try_with_handler;

pub use insta;
pub use pretty_assertions;

pub fn print_one<N: Node>(
  node: &N,
  cm: Option<Lrc<SourceMap>>,
  conf: Option<Config>,
) -> Result<String, anyhow::Error> {
  let cm = cm.unwrap_or_else(|| Lrc::new(SourceMap::default()));
  let mut buf = vec![];
  let mut emitter = Emitter {
    cfg: conf.unwrap_or_default(),
    cm: cm.clone(),
    comments: None,
    wr: Box::new(JsWriter::new(cm, "\n", &mut buf, None)),
  };
  node.emit_with(&mut emitter)?;
  Ok(String::from_utf8_lossy(&buf[..]).to_string())
}

pub fn print_one_unchecked<N: Node>(node: &N) -> String {
  print_one(node, None, None).unwrap()
}

pub struct PrintExpr<'a>(pub &'a Expr);

impl Debug for PrintExpr<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("``````js\n")?;
    f.write_str(&print_one_unchecked(self.0))?;
    f.write_str("\n``````")
  }
}

impl PartialEq for PrintExpr<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.0.eq_ignore_span(other.0)
  }
}

#[macro_export]
macro_rules! assert_eq_codegen {
  ($left:expr, $right:expr) => {
    $crate::pretty_assertions::assert_eq!($crate::PrintExpr($left), $crate::PrintExpr($right));
  };
}

pub fn parse_one<N, F>(
  source: &str,
  syntax: Option<Syntax>,
  mut parse_fn: F,
) -> Result<N, anyhow::Error>
where
  N: Node,
  F: FnMut(
    &SourceFile,
    Syntax,
    EsVersion,
    Option<&dyn Comments>,
    &mut Vec<ecma::parser::error::Error>,
  ) -> Result<N, ecma::parser::error::Error>,
{
  let sourcemap: Lrc<SourceMap> = Default::default();
  let source = sourcemap.new_source_file(FileName::Anon, source.to_string());

  let syntax = syntax.unwrap_or_default();
  let target = EsVersion::latest();
  let mut errors: Vec<ecma::parser::error::Error> = vec![];

  try_with_handler(sourcemap.clone(), Default::default(), |reporter| {
    parse_fn(&source, syntax, target, None, &mut errors)
      .and_then(|result| {
        for err in errors {
          err.into_diagnostic(&reporter).emit();
        }
        Ok(result)
      })
      .map_err(|err| {
        err.into_diagnostic(&reporter).emit();
        anyhow::anyhow!("trying to parse string as valid ECMAScript")
      })
  })
}

pub fn parse_expr_unchecked(source: &str) -> Expr {
  *parse_one(source, None, parse_file_as_expr).unwrap()
}

pub fn test_fixture<Config, Parse, Transform, Folder>(
  source_path: PathBuf,
  parse: Parse,
  transform: Transform,
) where
  Config: DeserializeOwned + Default,
  Parse: FnOnce(Lrc<SourceFile>) -> Module,
  Transform: FnOnce(Config) -> Folder,
  Folder: Fold,
{
  let config_path = source_path.clone().with_extension("json");
  let config: Config = std::fs::read_to_string(config_path)
    // exits on deserialize error
    .and_then(|s| match serde_json::from_str(&s) {
      Ok(v) => Ok(v),
      Err(e) => {
        panic!("Error: {}", e);
      }
    })
    // default on file not found
    .unwrap_or_default();

  let sourcemap: Lrc<SourceMap> = Default::default();
  let source = sourcemap.new_source_file(
    FileName::Anon,
    std::fs::read_to_string(source_path.clone()).unwrap(),
  );

  let mut transform = transform(config);

  let module = parse(source.clone());

  let module = module.fold_with(&mut transform);

  let mut code = vec![];
  let mut emitter = Emitter {
    cfg: Default::default(),
    cm: sourcemap.clone(),
    comments: None,
    wr: JsWriter::new(sourcemap.clone(), "\n", &mut code, None),
  };
  module.emit_with(&mut emitter).unwrap();

  let mut result = String::new();
  result.push_str("``````js\n");
  result.push_str(&String::from_utf8_lossy(&code).trim());
  result.push_str("\n``````");

  let snapshot_path = source_path.parent().unwrap();

  let snapshot_name = source_path
    .file_name()
    .unwrap()
    .to_string_lossy()
    .to_string();

  insta::with_settings!({
    snapshot_path => snapshot_path,
    prepend_module_to_snapshot => false,
  }, {
    insta::assert_snapshot!(snapshot_name, result);
  });
}

#[cfg(test)]
mod tests {
  use swc_core::ecma::parser::{parse_file_as_expr, parse_file_as_module};

  use super::{parse_one, print_one};

  #[test]
  fn test_parse_print_expr() {
    let source = "42";
    let parsed = parse_one(source, None, parse_file_as_expr).unwrap();
    let output = print_one(&parsed, None, None).unwrap();
    assert_eq!(source, output);
  }

  #[test]
  fn test_parse_print_module() {
    let source = "export const foo = 42;";
    let parsed = parse_one(source, None, parse_file_as_module).unwrap();
    let output = print_one(&parsed, None, None).unwrap();
    assert_eq!(source.trim(), output.trim());
  }

  #[test]
  #[should_panic(expected = "Unexpected eof")]
  fn test_parse_recoverable_error() {
    let source = "{";
    let _ = parse_one(source, None, parse_file_as_expr).unwrap();
  }

  #[test]
  #[should_panic(expected = "Expected ';', '}' or <eof>")]
  fn test_parse_error() {
    let source = "a b";
    let _ = parse_one(source, None, parse_file_as_module).unwrap();
  }
}
