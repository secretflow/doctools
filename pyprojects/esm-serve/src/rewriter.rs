use std::collections::HashMap;

use minijinja::{context, Environment, Template};
use serde::{Deserialize, Serialize};
use swc_core::ecma::{
    ast::ImportDecl,
    visit::{as_folder, Fold, VisitMut},
};

use crate::specifier::{is_bare_specifier, parse_specifier};

#[derive(Serialize, Deserialize)]
pub struct ExternalPackageOptions {
    pub import_source: String,
    pub known_packages: HashMap<String, String>,
}

struct ModuleSpecifierRewriter<'source> {
    options: &'source ExternalPackageOptions,
    env: Environment<'source>,
}

impl<'source> ModuleSpecifierRewriter<'source> {
    fn new(options: &'source ExternalPackageOptions) -> Self {
        let mut env = Environment::new();
        env.add_template("import_source", &options.import_source)
            .unwrap();
        Self { env, options }
    }

    fn get_template(&'source self) -> Template<'source, 'source> {
        self.env.get_template("import_source").unwrap()
    }

    fn rewrite_specifier(&self, specifier: &str) -> Option<String> {
        if !is_bare_specifier(specifier) {
            return None;
        };

        let import = parse_specifier(specifier)?;

        let any_version = String::from(">=0.0.0");

        let version = self
            .options
            .known_packages
            .get(import.package)
            .unwrap_or(&any_version);

        let template = self.get_template();

        template
            .render(context! {
                package => import.package,
                version => version,
                path => import.path,
            })
            .ok()
    }
}

impl VisitMut for ModuleSpecifierRewriter<'_> {
    fn visit_mut_import_decl(&mut self, import: &mut ImportDecl) {
        let source = import.src.value.as_str();
        match self.rewrite_specifier(source) {
            Some(rewritten) => {
                import.src.value = rewritten.into();
                import.src.raw = None;
            }
            None => {}
        };
    }
}

pub fn externalize_modules(options: &ExternalPackageOptions) -> impl Fold + '_ {
    as_folder(ModuleSpecifierRewriter::new(options))
}
