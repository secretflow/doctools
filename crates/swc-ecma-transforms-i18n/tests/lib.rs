use std::path::PathBuf;

use ansi_term::Color;
use swc_core::{
  common::{input::StringInput, sync::Lrc, FileName, SourceMap},
  ecma::{
    codegen::{text_writer::JsWriter, Emitter, Node as _},
    parser::{lexer::Lexer, Parser, Syntax},
    visit::VisitMutWith as _,
  },
  testing::{diff, fixture},
};

use swc_ecma_transforms_i18n::{Translator, TranslatorOptions};

#[fixture("tests/fixtures/*.in.js")]
fn test_i18n(input: PathBuf) {
  let options: TranslatorOptions = std::fs::read_to_string(input.clone().with_extension("json"))
    .and_then(|s| match serde_json::from_str(&s) {
      Ok(v) => Ok(v),
      Err(e) => {
        panic!("Error: {}", e);
      }
    })
    .unwrap_or_default();

  let expected = std::fs::read_to_string(
    input
      .clone()
      .with_extension("")
      .with_extension("swc-snapshot.js"),
  )
  .unwrap();
  let expected = expected.trim();

  let sourcemap: Lrc<SourceMap> = Default::default();
  let source = sourcemap.new_source_file(
    FileName::Anon,
    std::fs::read_to_string(input.clone()).unwrap(),
  );

  let lexer = Lexer::new(
    Syntax::Es(Default::default()),
    Default::default(),
    StringInput::from(&*source),
    None,
  );

  let mut parser = Parser::new_from(lexer);

  for e in parser.take_errors() {
    panic!("{:?}", e)
  }

  let mut module = parser
    .parse_module()
    .map_err(|e| panic!("{:?}", e))
    .unwrap();

  let jsx = Default::default();
  let mut messages = vec![];

  let mut translator = Translator::new(jsx, options, &mut messages);

  module.visit_mut_children_with(&mut translator);

  let mut code = vec![];

  let mut emitter = Emitter {
    cfg: Default::default(),
    cm: sourcemap.clone(),
    comments: None,
    wr: JsWriter::new(sourcemap.clone(), "\n", &mut code, None),
  };

  module.emit_with(&mut emitter).unwrap();

  let actual = String::from_utf8(code).unwrap();
  let actual = actual.trim();

  if actual == expected {
    return;
  }

  print!(
    ">>>>> {} <<<<<\n\n{}\n\n",
    Color::Blue.paint("Source"),
    Color::Blue.paint(source.src.as_str()),
  );
  print!(
    ">>>>> {} <<<<<\n\n{}\n\n",
    Color::Yellow.paint("Transformed"),
    Color::Yellow.paint(actual),
  );

  if actual != expected {
    panic!(
      "assertion failed (actual != expected)\n{}",
      diff(&actual, expected)
    )
  }
}
