use std::path::PathBuf;

use ansi_term::Color;
use anyhow::Result;
use swc_core::{
    common::{sync::Lrc, SourceMap},
    ecma::{
        ast::{EsVersion, ExprStmt, Module, ModuleItem, Stmt},
        codegen::{text_writer::JsWriter, Config, Emitter},
    },
};
use swc_utils::jsx::factory::JSXFactory;
use testing::{diff, fixture};

use html5jsx::{html_to_jsx, Fragment};

fn make_module(f: Fragment) -> Module {
    let mut body = f
        .head
        .into_iter()
        .map(|e| {
            ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                expr: e,
                span: Default::default(),
            }))
        })
        .collect::<Vec<_>>();
    body.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        expr: f.body,
        span: Default::default(),
    })));
    Module {
        body,
        shebang: None,
        span: Default::default(),
    }
}

fn compile(module: &Module) -> Result<String> {
    let mut output = vec![];
    let cm: Lrc<SourceMap> = Default::default();
    let mut emitter = Emitter {
        cfg: Config::default().with_target(EsVersion::EsNext),
        cm: cm.clone(),
        comments: None,
        wr: JsWriter::new(cm.clone(), "\n", &mut output, None),
    };
    emitter.emit_module(&module)?;
    Ok(String::from_utf8(output)?)
}

#[fixture("tests/fixtures/expects/*.html")]
fn test_conversion(input: PathBuf) {
    let jsx: JSXFactory = std::fs::read_to_string(input.clone().with_extension("json"))
        // exits on deserialize error
        .and_then(|s| match serde_json::from_str(&s) {
            Ok(v) => Ok(v),
            Err(e) => {
                panic!("Error: {}", e);
            }
        })
        // default on file not found
        .unwrap_or_default();

    let expected = std::fs::read_to_string(input.clone().with_extension("js")).unwrap();
    let expected = expected.trim();

    let source = std::fs::read_to_string(input.clone()).unwrap();
    let fragment = html_to_jsx(&source, Some(jsx)).unwrap();

    let actual = compile(&make_module(fragment)).unwrap();
    let actual = actual.trim();

    if actual == expected {
        return;
    }

    print!(
        ">>>>> {} <<<<<\n\n{}\n\n",
        Color::Green.paint("Orig"),
        source
    );
    print!(
        ">>>>> {} <<<<<\n\n{}\n\n",
        Color::Green.paint("Code"),
        actual
    );

    if actual != expected {
        panic!(
            "assertion failed (actual != expected)\n{}",
            diff(actual, expected)
        )
    }
}

#[cfg(test)]
mod test_rejections {
    use html5jsx::html_to_jsx;
    use swc_utils::jsx::factory::JSXFactory;

    #[test]
    #[should_panic = "refuse to parse script tags"]
    fn no_unsafe_inline() {
        html_to_jsx("<script>alert('Hi!');</script>", None).unwrap();
    }

    #[test]
    #[should_panic = "refuse to parse script tags"]
    fn no_remote_script() {
        html_to_jsx(
            r#"<script src="https://cdn.jsdelivr.net/npm/lodash@4.17.21/lodash.min.js"></script>"#,
            None,
        )
        .unwrap();
    }

    #[test]
    #[should_panic = "refuse to parse base tags"]
    fn no_base() {
        html_to_jsx("<base href='https://example.com/' />", None).unwrap();
    }

    #[test]
    #[should_panic = "refuse to convert event handlers"]
    fn no_on_click() {
        html_to_jsx("<div onclick='alert(\"Hi!\")'>Hi!</div>", None).unwrap();
    }

    #[test]
    #[should_panic = "refuse to convert event handlers"]
    fn no_arbitrary_event_handlers() {
        html_to_jsx("<div onfoo='alert(\"Hi!\")'>Hi!</div>", None).unwrap();
    }

    #[test]
    #[should_panic = "refuse to convert dangerouslySetInnerHTML"]
    fn no_dangerously_set_inner_html() {
        html_to_jsx(
            "<div dangerouslySetInnerHTML={{__html: '<script>alert(\"Hi!\")</script>'}}></div>",
            None,
        )
        .unwrap();
    }

    #[test]
    #[should_panic = "refuse to convert `javascript:` URLs"]
    fn no_javascript_url() {
        html_to_jsx("<a href='javascript:alert(\"Hi!\")'>Hi!</a>", None).unwrap();
    }

    #[test]
    #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
    fn no_malicious_jsx() {
        html_to_jsx(
            "<div>",
            Some(JSXFactory {
                jsx: "eval".into(),
                ..Default::default()
            }),
        )
        .unwrap();
    }

    #[test]
    #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
    fn no_malicious_jsx_2() {
        html_to_jsx(
            "<div>",
            Some(JSXFactory {
                jsx: "evaluate".into(),
                ..Default::default()
            }),
        )
        .unwrap();
    }

    #[test]
    #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
    fn no_malicious_jsxs() {
        html_to_jsx(
            "<div>",
            Some(JSXFactory {
                jsxs: "globalThis.eval".into(),
                ..Default::default()
            }),
        )
        .unwrap();
    }

    #[test]
    #[should_panic = "JSX factories cannot contain 'eval' or 'Function' in name"]
    fn no_malicious_fragment() {
        html_to_jsx(
            "<div>",
            Some(JSXFactory {
                fragment: "window.Function".into(),
                ..Default::default()
            }),
        )
        .unwrap();
    }
}