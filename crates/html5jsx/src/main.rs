use std::{io::Read as _, vec};

use html5jsx::html_to_jsx;
use swc_core::ecma::ast::{
    BlockStmt, DefaultDecl, ExportDefaultDecl, FnExpr, Function, Module, ModuleItem, ReturnStmt,
    Stmt,
};
use swc_utils::{print_one, JSXFactory};

fn main() {
    // read HTML from stdin
    let mut html = String::new();
    std::io::stdin().read_to_string(&mut html).unwrap();

    // parse
    let jsx = JSXFactory::default();
    let fragment = html_to_jsx(&html, Some(jsx.clone())).unwrap();

    let html = jsx.create(
        "html".into(),
        None,
        Some(vec![
            jsx.create("head".into(), None, Some(fragment.head)),
            jsx.create("body".into(), None, Some(vec![fragment.body])),
        ]),
    );

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
            ModuleItem::ModuleDecl(jsx.clone().import_from("react/jsx-runtime").into()),
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

    // print
    let output = print_one(&module, Default::default());
    println!("{}", output);
}
