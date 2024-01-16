use std::{io::Read as _, vec};

use base64::{prelude::BASE64_STANDARD, Engine};
use swc_core::{
    common::{sync::Lrc, FileName, SourceMap},
    ecma::{
        ast::{
            BlockStmt, DefaultDecl, ExportDefaultDecl, FnExpr, Function, Module, ModuleItem,
            ReturnStmt, Stmt,
        },
        codegen::{text_writer::JsWriter, Emitter},
    },
};

use html5jsx::html_to_jsx;
use swc_utils::jsx::factory::JSXFactory;

fn main() {
    // read HTML from stdin
    let mut html = String::new();
    std::io::stdin().read_to_string(&mut html).unwrap();

    let sourcemap = Lrc::new(SourceMap::default());
    let source = sourcemap.new_source_file(FileName::Anon, html);

    // parse
    let jsx = JSXFactory::default();
    let fragment = html_to_jsx(&source, Some(jsx.clone())).unwrap();

    let html = jsx
        .create(&"html".into())
        .children(Some(vec![
            jsx.create(&"head".into())
                .children(Some(fragment.head))
                .build(),
            jsx.create(&"body".into())
                .children(Some(vec![fragment.body]))
                .build(),
        ]))
        .build();

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
