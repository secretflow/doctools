use anyhow::Result;
use swc_core::{
    self,
    common::{
        comments::SingleThreadedComments,
        errors::{ColorConfig, Handler},
        sync::Lrc,
        FileName, SourceMap,
    },
    ecma::{
        ast::EsVersion,
        codegen::{text_writer::JsWriter, Config, Emitter},
        parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax},
        visit::FoldWith,
    },
};

mod rewriter;
mod specifier;

use crate::rewriter::{externalize_modules, ExternalPackageOptions};

fn main() -> Result<()> {
    let files: Lrc<SourceMap> = Default::default();
    let console = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(files.clone()));

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    let source = files.new_source_file(FileName::Custom("<stdin>".into()), buffer);
    let comments = SingleThreadedComments::default();

    let lexer = Lexer::new(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        EsVersion::EsNext,
        StringInput::from(&*source),
        Some(&comments),
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&console).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| e.into_diagnostic(&console).emit())
        .expect("failed to parse module");

    let module = module.fold_with(&mut externalize_modules(&ExternalPackageOptions {
        import_source: "https://cdn.jsdelivr.net/npm/{{package}}@{{version}}{{path}}/+esm".into(),
        // import_source: "https://esm.sh/{{package}}@{{version}}{{path}}".into(),
        known_packages: vec![
            ("react".into(), "^18.2.0".into()),
            ("react-dom".into(), "^18.2.0".into()),
        ]
        .into_iter()
        .collect(),
    }));

    let mut output = vec![];
    let mut emitter = Emitter {
        cfg: Config::default().with_target(EsVersion::EsNext),
        cm: files.clone(),
        comments: Some(&comments),
        wr: JsWriter::new(files.clone(), "\n", &mut output, None),
    };
    emitter.emit_module(&module).unwrap();
    println!("{}", String::from_utf8(output)?);

    Ok(())
}
