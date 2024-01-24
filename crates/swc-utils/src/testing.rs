use swc_core::{
  common::{comments::Comments, sync::Lrc, FileName, SourceFile, SourceMap},
  ecma::{
    self,
    ast::EsVersion,
    codegen::{text_writer::JsWriter, Config, Emitter, Node},
    parser::Syntax,
  },
};
use swc_error_reporters::handler::try_with_handler;

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
