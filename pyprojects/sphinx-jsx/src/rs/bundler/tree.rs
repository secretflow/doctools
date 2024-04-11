use fuzzy_sourcemap::FuzzySourceMap;
use pyo3::prelude::*;
use serde::de::Error;
use swc_core::{
  common::{sync::Lrc, FileName, SourceMap},
  ecma::ast::{Expr, ObjectLit},
};
use swc_ecma_utils2::{
  ad_hoc_tag,
  ecma::repack_expr,
  jsx::{DocumentBuilder, JSXDocument},
};

use super::{
  env::{Environment, SphinxFileLoader},
  symbols::Symbols,
};

#[pyclass]
pub struct Doctree {
  file: FileName,
  tree: DocumentBuilder<Symbols>,
  mapper: FuzzySourceMap,
  current_line: Option<usize>,
}

impl From<(Lrc<SourceMap>, Lrc<Environment>, FileName)> for Doctree {
  fn from((files, env, file): (Lrc<SourceMap>, Lrc<Environment>, FileName)) -> Self {
    Self {
      file,
      tree: Default::default(),
      mapper: FuzzySourceMap::new(files, Box::new(SphinxFileLoader::from(env))),
      current_line: None,
    }
  }
}

#[pymethods]
impl Doctree {
  #[pyo3(signature = (tag, attrs, *, file=None, line=None, source=None))]
  fn element(
    &mut self,
    tag: &str,
    attrs: &str,
    file: Option<&str>,
    line: Option<usize>,
    source: Option<&str>,
  ) -> anyhow::Result<()> {
    let line = line.or(self.current_line);

    let props: serde_json::Value = serde_json::from_str(attrs)?;

    if !matches!(props, serde_json::Value::Object(_)) {
      return Err(serde_json::Error::custom("element attributes must be an object").into());
    }

    let source = if reject_source_text(tag, &props) {
      None
    } else {
      source
    };

    let span = self.mapper.feed(file, line, source).unwrap_or_default();

    if !span.is_dummy() && line.is_some() {
      self.current_line = line;
    }

    let props: ObjectLit = repack_expr(Default::default(), &props)
      .expect("serializing JSON to AST should never fail")
      .object()
      .expect("props should always be an object");

    self.tree.element(span, ad_hoc_tag!(<> tag), props);

    Ok(())
  }

  fn enter(&mut self) {
    self.tree.enter(&["children"]);
  }

  fn text(&mut self, text: &str) {
    let span = self.mapper.feed(None, None, Some(text)).unwrap_or_default();
    let text: Expr = repack_expr(span, &text).expect("serializing text to AST should never fail");
    self.tree.value(text);
  }

  fn exit(&mut self) {
    self.tree.exit();
  }
}

impl Doctree {
  pub fn close(&mut self) -> anyhow::Result<(FileName, JSXDocument)> {
    let file = std::mem::replace(&mut self.file, FileName::Anon);

    if !matches!(file, FileName::Real(_)) {
      return Err(anyhow::anyhow!("doctree already closed"));
    }

    let mut tree = std::mem::take(&mut self.tree);

    tree.flush();

    Ok((file, tree.into_document()))
  }
}

fn reject_source_text(tag: &str, props: &serde_json::Value) -> bool {
  match tag {
    "footnote_reference" => props.get("auto").is_some(),
    _ => false,
  }
}
