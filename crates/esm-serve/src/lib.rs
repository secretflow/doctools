use std::{collections::HashMap, fmt::Display};

use minijinja::{Environment, Template};
use serde::{Deserialize, Serialize};
use swc_core::{
    common::{errors::HANDLER, Span},
    ecma::{
        ast::{CallExpr, ExportAll, ImportDecl, Lit, NamedExport},
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
    },
};
use url::Url;

mod specifier;

use crate::specifier::{is_bare_specifier, parse_specifier};

#[derive(Debug)]
pub enum CannotRewriteSpecifier {
    RenderingError(minijinja::Error),
    InvalidURL(url::ParseError),
}

impl Display for CannotRewriteSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CannotRewriteSpecifier::*;
        match self {
            RenderingError(err) => write!(
                f,
                "cannot render module specifier using minijinja: {:#?}",
                err
            ),
            InvalidURL(err) => write!(f, "module rewrite resulted in an invalid URL: {:#?}", err),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalPackages {
    /// The template to use for rewriting module specifiers,
    /// rendered with the following context:
    ///
    /// - `package`
    /// - `version`
    /// - `path`: The imported path within the package (leading slash included)
    ///
    /// The default template is `{{package}}{{path}}`, which does nothing.
    ///
    /// ### Examples
    ///
    /// - `https://esm.sh/{{package}}@{{version}}{{path}}`
    /// - `https://deno.land/x/{{package}}@{{version}}{{path}}`
    #[serde(default = "ExternalPackages::noop_import_source")]
    import_source: String,

    /// A map of package names to versions.
    ///
    /// If a package is not in this map, it is assumed to be `>=0.0.0`.
    #[serde(default)]
    known_packages: HashMap<String, String>,
}

impl ExternalPackages {
    fn noop_import_source() -> String {
        String::from("{{package}}{{path}}")
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn import_from(mut self, source: &str) -> Self {
        self.import_source = source.to_string();
        self
    }

    pub fn package(mut self, package: &str, version: &str) -> Self {
        self.known_packages
            .insert(package.to_string(), version.to_string());
        self
    }
}

#[derive(Serialize)]
struct ModuleInfo<'a> {
    package: &'a str,
    version: &'a str,
    path: &'a str,
}

struct ModuleSpecifierRewriter<'options> {
    packages: &'options ExternalPackages,
    jinja: Environment<'options>,
}

impl<'options> ModuleSpecifierRewriter<'options> {
    fn new(packages: &'options ExternalPackages) -> Self {
        let mut jinja: Environment<'options> = Environment::new();

        match jinja.add_template("import_source", &packages.import_source) {
            Ok(_) => (),
            Err(err) => HANDLER.with(|handler| {
                handler
                    .struct_err(&format!(
                        "[esm-serve] cannot parse import source: {:#?}",
                        err
                    ))
                    .emit();
                panic!()
            }),
        };

        Self { packages, jinja }
    }

    fn get_template(&'options self) -> Template<'options, 'options> {
        self.jinja.get_template("import_source").unwrap()
    }

    fn rewrite_specifier(&self, specifier: &str) -> Result<Option<String>, CannotRewriteSpecifier> {
        if !is_bare_specifier(specifier) {
            return Ok(None);
        };

        let import = match parse_specifier(specifier) {
            None => return Ok(None),
            Some(import) => import,
        };

        let any_version = String::from(">=0.0.0");

        let template = self.get_template();

        let rewritten = template
            .render(ModuleInfo {
                package: import.package,
                version: self
                    .packages
                    .known_packages
                    .get(import.package)
                    .unwrap_or(&any_version),
                path: import.path,
            })
            .map_err(CannotRewriteSpecifier::RenderingError)?;

        Url::parse(&rewritten)
            .map(|u| Some(u.to_string()))
            .or_else(|err| match err {
                url::ParseError::RelativeUrlWithoutBase => Ok(Some(rewritten)),
                _ => Err(CannotRewriteSpecifier::InvalidURL(err)),
            })
    }

    fn emit_error(&self, span: Span, err: CannotRewriteSpecifier) {
        HANDLER.with(|handler| {
            handler
                .struct_span_err(span, &format!("[esm-serve] {}", err))
                .note(&format!("template was {:?}", self.packages.import_source))
                .emit()
        });
    }
}

impl VisitMut for ModuleSpecifierRewriter<'_> {
    noop_visit_mut_type!();

    /// import { thing } from "module"
    /// import * as thing from "module"
    /// import thing from "module"
    /// import thing from "package.json" assert { type: "json" }
    fn visit_mut_import_decl(&mut self, import: &mut ImportDecl) {
        let source = import.src.value.as_str();
        match self.rewrite_specifier(source) {
            Ok(Some(rewritten)) => {
                import.src.value = rewritten.into();
                import.src.raw = None;
            }
            Ok(None) => (),
            Err(err) => {
                self.emit_error(import.span, err);
            }
        };
    }

    /// import("module")
    fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
        if call.callee.as_import().is_none() {
            return;
        }

        let arg0 = match call.args.first_mut() {
            Some(arg) => arg,
            None => return,
        };

        let specifier = match arg0.expr.as_lit() {
            Some(Lit::Str(str)) => str.value.as_str(),
            _ => return,
        };

        match self.rewrite_specifier(specifier) {
            Ok(Some(rewritten)) => {
                arg0.expr = rewritten.into();
            }
            Ok(None) => (),
            Err(err) => {
                self.emit_error(call.span, err);
            }
        }
    }

    /// export * as thing from "module"
    /// export { thing } from "module"
    /// export { thing as other } from "module"
    fn visit_mut_named_export(&mut self, export: &mut NamedExport) {
        let source = match &export.src {
            Some(src) => src.value.as_str(),
            None => return,
        };
        match self.rewrite_specifier(source) {
            Ok(Some(rewritten)) => {
                export.src = Some(Box::new(rewritten.into()));
            }
            Ok(None) => (),
            Err(err) => {
                self.emit_error(export.span, err);
            }
        };
    }

    /// export * from "module"
    fn visit_mut_export_all(&mut self, export: &mut ExportAll) {
        let source = export.src.value.as_str();
        match self.rewrite_specifier(source) {
            Ok(Some(rewritten)) => {
                export.src.value = rewritten.into();
                export.src.raw = None;
            }
            Ok(None) => (),
            Err(err) => {
                self.emit_error(export.span, err);
            }
        };
    }
}

pub fn externalize_modules(options: &ExternalPackages) -> impl Fold + VisitMut + '_ {
    as_folder(ModuleSpecifierRewriter::new(options))
}
