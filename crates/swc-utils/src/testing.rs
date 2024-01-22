use swc_core::{
  common::{input::StringInput, sync::Lrc, FileName, SourceMap},
  ecma::{
    self,
    ast::EsVersion,
    codegen::{text_writer::JsWriter, Config, Emitter, Node},
    parser::{lexer::Lexer, Parser, Syntax},
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
  F: FnMut(&mut Parser<Lexer<'_>>) -> Result<N, ecma::parser::error::Error>,
{
  let sourcemap: Lrc<SourceMap> = Default::default();
  let source = sourcemap.new_source_file(FileName::Anon, source.to_string());

  let lexer = Lexer::new(
    syntax.unwrap_or_default(),
    EsVersion::latest(),
    StringInput::from(&*source),
    None,
  );

  try_with_handler(sourcemap.clone(), Default::default(), |reporter| {
    let mut parser = Parser::new_from(lexer);
    parse_fn(&mut parser)
      .and_then(|result| {
        for err in parser.take_errors() {
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
  use super::{parse_one, print_one};

  #[test]
  fn test_parse_print_expr() {
    let source = "42";
    let parsed = parse_one(source, None, |p| p.parse_expr()).unwrap();
    let output = print_one(&parsed, None, None).unwrap();
    assert_eq!(source, output);
  }

  #[test]
  fn test_parse_print_module() {
    let source = "export const foo = 42;";
    let parsed = parse_one(source, None, |p| p.parse_module()).unwrap();
    let output = print_one(&parsed, None, None).unwrap();
    assert_eq!(source.trim(), output.trim());
  }

  #[test]
  #[should_panic(expected = "Expected ';', '}' or <eof>")]
  fn test_parse_error() {
    let source = "a b";
    let _ = parse_one(source, None, |p| p.parse_module()).unwrap();
  }
}
