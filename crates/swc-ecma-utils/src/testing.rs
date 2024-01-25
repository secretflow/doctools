use std::path::PathBuf;

use serde::de::DeserializeOwned;
use swc_core::{
  common::{comments::Comments, sync::Lrc, FileName, SourceFile, SourceMap},
  ecma::{
    self,
    ast::{
      ArrayLit, Decl, EsVersion, ExportDecl, Expr, ExprOrSpread, Ident, Module, ModuleDecl,
      ModuleItem, VarDecl, VarDeclKind, VarDeclarator,
    },
    codegen::{text_writer::JsWriter, Config, Emitter, Node},
    parser::Syntax,
    visit::{Fold, FoldWith as _},
  },
  testing::diff,
};
use swc_error_reporters::handler::try_with_handler;

use crate::jsx::{builder::JSXDocument, factory::JSXRuntime};

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

pub fn print_one_unwrap<N: Node>(node: &N) -> String {
  print_one(node, None, None).unwrap()
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

fn get_snapshot_path(source_path: PathBuf) -> PathBuf {
  let mut source_path = source_path;
  loop {
    if source_path.extension().is_some() {
      source_path = source_path.with_extension("");
    } else {
      break;
    }
  }
  source_path.with_extension("swc-snapshot.js")
}

pub fn test_fixture<Config, Parse, Transform, Folder>(
  source_path: PathBuf,
  parse: Parse,
  transform: Transform,
) where
  Config: DeserializeOwned + Default,
  Parse: FnOnce(Lrc<SourceFile>) -> Module,
  Transform: FnOnce(Lrc<JSXRuntime>, Config) -> Folder,
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

  let runtime = Lrc::new(JSXRuntime::default());

  let snapshot_path = get_snapshot_path(source_path.clone());
  let snapshot = std::fs::read_to_string(snapshot_path).unwrap();
  let snapshot = snapshot.trim();

  let sourcemap: Lrc<SourceMap> = Default::default();
  let source = sourcemap.new_source_file(
    FileName::Anon,
    std::fs::read_to_string(source_path.clone()).unwrap(),
  );

  let mut transform = transform(runtime, config);

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

  let actual = String::from_utf8(code).unwrap();
  let actual = actual.trim();

  if actual == snapshot {
    return;
  }

  print!(">>>>> Source <<<<<\n\n{}\n\n", source.src.as_str());
  print!(">>>>> Transformed <<<<<\n\n{}\n\n", actual);
  panic!(
    "assertion failed (actual != expected)\n{}",
    diff(&actual, snapshot)
  )
}

pub fn document_as_module(mut document: JSXDocument) -> Module {
  Module {
    span: Default::default(),
    body: vec![ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
      span: Default::default(),
      decl: Decl::Var(
        VarDecl {
          span: Default::default(),
          kind: VarDeclKind::Const,
          declare: false,
          decls: vec![
            VarDeclarator {
              definite: false,
              span: Default::default(),
              name: Ident::from("head").into(),
              init: Some(
                Expr::from(ArrayLit {
                  elems: document
                    .head
                    .drain(..)
                    .map(|e| {
                      Some(ExprOrSpread {
                        expr: e,
                        spread: None,
                      })
                    })
                    .collect(),
                  span: Default::default(),
                })
                .into(),
              ),
            },
            VarDeclarator {
              definite: false,
              span: Default::default(),
              name: Ident::from("body").into(),
              init: Some(
                Expr::from(ArrayLit {
                  elems: document
                    .body
                    .drain(..)
                    .map(|e| {
                      Some(ExprOrSpread {
                        expr: e,
                        spread: None,
                      })
                    })
                    .collect(),
                  span: Default::default(),
                })
                .into(),
              ),
            },
          ],
        }
        .into(),
      ),
    }))],
    shebang: None,
  }
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
