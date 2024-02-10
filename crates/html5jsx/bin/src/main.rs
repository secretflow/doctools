use std::{io::Read as _, vec};

use base64::{prelude::BASE64_STANDARD, Engine};
use swc_core::{
  common::{sync::Lrc, FileName, SourceMap},
  ecma::{
    ast::{
      BlockStmt, DefaultDecl, ExportDefaultDecl, FnExpr, Function, Ident, ImportDecl,
      ImportNamedSpecifier, ImportSpecifier, Module, ModuleItem, ReturnStmt, Stmt, Str,
    },
    codegen::{text_writer::JsWriter, Emitter},
  },
};

use html5jsx::html_to_jsx;
use swc_ecma_utils2::jsx::factory::JSXRuntime;

pub fn import_from(factory: &JSXRuntime, src: &str) -> ImportDecl {
  ImportDecl {
    specifiers: vec![
      ImportSpecifier::Named(ImportNamedSpecifier {
        local: Ident::from(&*factory.jsx().as_expr().unwrap().as_ident().unwrap().sym),
        imported: None,
        is_type_only: false,
        span: Default::default(),
      }),
      ImportSpecifier::Named(ImportNamedSpecifier {
        local: Ident::from(&*factory.jsxs().as_expr().unwrap().as_ident().unwrap().sym),
        imported: None,
        is_type_only: false,
        span: Default::default(),
      }),
      ImportSpecifier::Named(ImportNamedSpecifier {
        local: Ident::from(
          &*factory
            .fragment()
            .as_expr()
            .unwrap()
            .as_ident()
            .unwrap()
            .sym,
        ),
        imported: None,
        is_type_only: false,
        span: Default::default(),
      }),
    ],
    src: Box::from(Str::from(src)),
    type_only: false,
    with: None,
    span: Default::default(),
    phase: Default::default(),
  }
}

fn main() {
  // read HTML from stdin
  let mut html = String::new();
  std::io::stdin().read_to_string(&mut html).unwrap();

  let sourcemap = Lrc::new(SourceMap::default());
  let source = sourcemap.new_source_file(FileName::Anon, html);

  // parse
  let jsx = JSXRuntime::default();
  let fragment = html_to_jsx(&source, Some(jsx.clone())).unwrap();

  let html = jsx
    .create(&"html".into())
    .children(vec![
      jsx
        .create(&"head".into())
        .children(fragment.head)
        .build()
        .into(),
      jsx
        .create(&"body".into())
        .children(fragment.body)
        .build()
        .into(),
    ])
    .build()
    .into();

  let main = FnExpr {
    ident: Some("App".into()),
    function: Function {
      body: Some(BlockStmt {
        stmts: vec![Stmt::from(ReturnStmt {
          arg: Some(html),
          span: Default::default(),
        })],
        span: Default::default(),
      }),
      params: vec![],
      decorators: vec![],
      is_generator: false,
      is_async: false,
      type_params: None,
      return_type: None,
      span: Default::default(),
    }
    .into(),
  };

  // build
  let module = Module {
    body: vec![
      ModuleItem::ModuleDecl(import_from(&jsx, "react/jsx-runtime").into()),
      ModuleItem::ModuleDecl(
        ExportDefaultDecl {
          decl: DefaultDecl::Fn(main.into()),
          span: Default::default(),
        }
        .into(),
      ),
    ],
    shebang: None,
    span: Default::default(),
  };

  let mut code = vec![];
  let mut srcmap = vec![];

  {
    let mut srcmap_raw = vec![];
    let mut emitter = Emitter {
      cfg: Default::default(),
      cm: sourcemap.clone(),
      comments: None,
      wr: JsWriter::new(sourcemap.clone(), "\n", &mut code, Some(&mut srcmap_raw)),
    };
    emitter.emit_module(&module).unwrap();
    sourcemap
      .build_source_map(&srcmap_raw)
      .to_writer(&mut srcmap)
      .unwrap();
  }

  let mut result = String::from_utf8(code).unwrap();
  result.push_str("\n//# sourceMappingURL=data:application/json;base64,");
  BASE64_STANDARD.encode_string(srcmap, &mut result);

  println!("{}", result);
}
