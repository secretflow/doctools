use deno_lite::ESModule;
use swc_core::{
  common::{sync::Lrc, FileName, SourceMap},
  ecma::{ast::Expr, visit::VisitMutWith as _},
};
use swc_ecma_transform_sphinx::{
  init_esm, render_code, render_math, render_raw, render_typography,
};
use swc_ecma_utils2::{
  ecma::fixes::remove_invalid,
  jsx::fixes::{fix_jsx_factories, fold_fragments},
};

use super::{env::Environment, lint::Linter, symbols::Symbols, Abort, Bundler};

pub struct Transformer {
  sourcemap: Lrc<SourceMap>,
  env: Lrc<Environment>,
  esm: ESModule,
  trees: Vec<(FileName, Expr)>,
}

impl Transformer {
  pub fn new(bundler: &Bundler, trees: Vec<(FileName, Expr)>) -> anyhow::Result<Self> {
    let esm = init_esm(bundler.deno.clone())?;

    Ok(Self {
      sourcemap: bundler.sourcemap.clone(),
      env: bundler.env.clone(),
      esm,
      trees,
    })
  }

  pub fn transform(mut self, abort: &Abort<'_>) -> anyhow::Result<Self> {
    self.trees.iter_mut().try_for_each(|(file, tree)| {
      abort.check()?;

      log::info!("transforming {}", self.env.docname(file));

      tree.visit_mut_with(&mut render_code::<Symbols>(
        self.sourcemap.clone(),
        &self.esm,
      ));

      tree.visit_mut_with(&mut render_math::<Symbols>(
        self.sourcemap.clone(),
        &self.esm,
      ));

      tree.visit_mut_with(&mut render_raw::<Symbols>(self.sourcemap.clone()));

      tree.visit_mut_with(&mut render_typography::<Symbols>());

      tree.visit_mut_with(&mut fold_fragments::<Symbols>());
      tree.visit_mut_with(&mut fix_jsx_factories::<Symbols>());
      tree.visit_mut_with(&mut remove_invalid());

      anyhow::Result::<()>::Ok(())
    })?;

    Ok(self)
  }

  pub fn into_linter(self, bundler: &Bundler) -> Linter {
    Linter::new(bundler, self.trees)
  }
}
