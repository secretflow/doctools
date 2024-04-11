use std::{io::Write, path::PathBuf};

use relative_path::RelativePathBuf;
use swc_core::{
  common::{source_map::SourceMapGenConfig, sync::Lrc, FileName, SourceMap, Spanned as _},
  ecma::{
    ast::{EsVersion, Expr, Lit, Str},
    codegen::{self, Node as _},
    visit::{noop_visit_type, Visit, VisitWith},
  },
};
use swc_ecma_utils2::jsx::JSXElement;
use url::Url;

use super::{diagnostics::ensure_char_boundary, symbols::Symbols, Bundler};

impl Bundler {
  fn emit_page_2(&self, module: Expr, src_path: PathBuf) -> anyhow::Result<()> {
    let src_url = Url::from_file_path(src_path.clone()).expect("should be a file URL");

    let rel_path = self.env.src_dir.make_relative(&src_url).ok_or_else(|| {
      anyhow::anyhow!(
        "unexpected source file {0} outside or source directory",
        src_path.to_string_lossy().to_string()
      )
    })?;

    let rel_path = RelativePathBuf::from(rel_path);

    let out_url = self.env.out_dir.join(rel_path.as_str())?;

    assert!(self.env.out_dir.make_relative(&out_url).is_some());

    let out_js = out_url
      .to_file_path()
      .expect("should be a file")
      .with_extension("js");
    let out_js_map = out_js.with_extension("js.map");
    let out_js_map_raw = out_js.with_extension("js.map.txt");
    let out_yaml = out_js.with_extension("yaml");

    std::fs::create_dir_all(out_js.parent().expect("should have a parent"))?;

    println!("Emitting {}", out_js.to_string_lossy());

    let mut out_js_file = std::fs::File::create(out_js)?;
    let mut out_js_map_file = std::fs::File::create(out_js_map)?;
    let mut out_js_map_raw_file = std::fs::File::create(out_js_map_raw)?;
    let out_yaml_file = std::fs::File::create(out_yaml)?;

    let mut source_mappings = vec![];

    {
      let mut emitter = codegen::Emitter {
        cfg: codegen::Config::default()
          .with_ascii_only(false)
          .with_minify(true)
          .with_target(EsVersion::latest()),
        cm: self.sourcemap.clone(),
        comments: None,
        wr: Box::new(codegen::text_writer::JsWriter::new(
          self.sourcemap.clone(),
          "\n",
          &mut out_js_file,
          Some(&mut source_mappings),
        )),
      };
      module.emit_with(&mut emitter)?;
    }

    let source_mappings = source_mappings
      .into_iter()
      .map(|(pos, loc)| ensure_char_boundary(&self.sourcemap, pos).map(|pos| (pos, loc)))
      .collect::<Result<Vec<_>, _>>()?;

    {
      out_js_map_raw_file.write_all(format!("{:#?}", source_mappings).as_bytes())?;
      serde_yaml::to_writer(out_yaml_file, &module)?;
    }

    {
      struct Config;

      impl SourceMapGenConfig for Config {
        fn file_name_to_source(&self, f: &FileName) -> String {
          f.to_string()
        }
        fn inline_sources_content(&self, _: &FileName) -> bool {
          true
        }
      }

      let source_map = self
        .sourcemap
        .build_source_map_with_config(&source_mappings, None, Config);
      source_map.to_writer(&mut out_js_map_file)?;
    }

    Ok(())
  }
}

fn debug_source_map(sources: Lrc<SourceMap>, module: &Expr) {
  struct SourceLocation(Lrc<SourceMap>);

  impl SourceLocation {
    fn maybe_emit(&self, expr: &Expr) -> Option<()> {
      let span = expr.span();
      if span.is_dummy() {
        return None;
      }
      let src = {
        let f = self.0.span_to_filename(span);
        self.0.get_source_file(&f)
      }?;
      let name = match expr {
        Expr::Call(call) => call.as_jsx_type::<Symbols>().map(|f| format!("{:?}", f)),
        Expr::Lit(Lit::Str(Str { value, .. })) => Some(value.to_string()),
        _ => None,
      }?;
      let report = miette::miette!(
        severity = miette::Severity::Advice,
        labels = vec![miette::LabeledSpan::new(
          Some(name),
          (span.lo().0 - src.start_pos.0) as usize,
          (span.hi().0 - span.lo().0) as usize,
        )],
        "source map"
      )
      .with_source_code(miette::NamedSource::new(
        src.name.to_string(),
        src.src.clone(),
      ));
      println!("{:?}", report);
      Some(())
    }
  }

  impl Visit for SourceLocation {
    noop_visit_type!();

    fn visit_expr(&mut self, expr: &Expr) {
      self.maybe_emit(expr);
      expr.visit_children_with(self);
    }
  }

  {
    module.visit_with(&mut SourceLocation(sources.clone()));
  }
}
