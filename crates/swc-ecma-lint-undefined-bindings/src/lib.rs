use std::collections::HashSet;

use swc_core::{
  atoms::Atom,
  ecma::{
    ast::{Decl, ExportDecl, Ident, MemberExpr, Pat, PropName, VarDeclarator},
    parser::{parse_file_as_module, Syntax, TsConfig},
    visit::{noop_visit_type, Visit, VisitAll, VisitAllWith, VisitWith},
  },
};

use swc_utils::testing::parse_one;

struct CollectExportDeclares {
  idents: HashSet<Atom>,
}

impl VisitAll for CollectExportDeclares {
  noop_visit_type!();

  fn visit_export_decl(&mut self, export: &ExportDecl) {
    match export.decl {
      Decl::Var(ref declare) => {
        if !declare.declare {
          return;
        }
        declare.decls.iter().for_each(|d| match d.name {
          Pat::Ident(ref ident) => {
            self.idents.insert((&*ident.id.sym).into());
          }
          _ => {}
        })
      }
      Decl::Class(ref declare) => {
        if !declare.declare {
          return;
        }
        self.idents.insert((&*declare.ident.sym).into());
      }
      Decl::Fn(ref declare) => {
        if !declare.declare {
          return;
        }
        self.idents.insert((&*declare.ident.sym).into());
      }
      _ => return,
    }
  }
}

pub struct CollectIdents {
  idents: HashSet<Atom>,
  defined: HashSet<Atom>,
}

impl Visit for CollectIdents {
  noop_visit_type!();

  fn visit_prop_name(&mut self, key: &PropName) {
    match key {
      PropName::Ident(_) => {}
      _ => key.visit_children_with(self),
    }
  }

  fn visit_member_expr(&mut self, member: &MemberExpr) {
    member.obj.visit_with(self);
  }

  fn visit_var_declarator(&mut self, decl: &VarDeclarator) {
    match decl.init {
      None => {
        decl.name.visit_with(self);
      }
      Some(ref init) => {
        let mut collector = CollectIdents {
          idents: HashSet::new(),
          defined: HashSet::new(),
        };
        decl.name.visit_with(&mut collector);
        self.defined.extend(collector.idents);
        init.visit_with(self)
      }
    };
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if self.defined.contains(&(&*ident.sym).into()) {
      return;
    }
    self.idents.insert((&*ident.sym).into());
  }
}

pub struct LintUndefinedBindings {
  declared: HashSet<Atom>,
}

impl LintUndefinedBindings {
  pub fn new(dts: Vec<String>) -> anyhow::Result<Self> {
    let mut declared = CollectExportDeclares {
      idents: HashSet::new(),
    };

    for src in dts {
      let ts = parse_one(
        &src.as_str(),
        Some(Syntax::Typescript(TsConfig {
          dts: true,
          ..Default::default()
        })),
        parse_file_as_module,
      )?;
      ts.visit_all_children_with(&mut declared);
    }

    Ok(Self {
      declared: declared.idents,
    })
  }

  /// ignore well-known symbols such as Object, Array, etc.
  ///
  /// this is never going to be exhaustive because we are not in the business of
  /// replacing ESLint.
  pub fn with_well_known_symbols(mut self) -> Self {
    self.declared.extend(vec![
      "window".into(),
      "document".into(),
      "console".into(),
      "location".into(),
      "history".into(),
      "navigator".into(),
      "atob".into(),
      "btoa".into(),
      "fetch".into(),
      "setTimeout".into(),
      "clearTimeout".into(),
      "setInterval".into(),
      "clearInterval".into(),
      "Promise".into(),
      "Object".into(),
      "Array".into(),
      "String".into(),
      "Number".into(),
      "Boolean".into(),
      "Symbol".into(),
      "Map".into(),
      "Set".into(),
      "WeakMap".into(),
      "WeakSet".into(),
      "Date".into(),
      "RegExp".into(),
      "Error".into(),
      "Math".into(),
      "JSON".into(),
      "parseInt".into(),
      "parseFloat".into(),
      "isNaN".into(),
      "isFinite".into(),
      "encodeURI".into(),
      "encodeURIComponent".into(),
      "decodeURI".into(),
      "NaN".into(),
      "Infinity".into(),
    ]);
    self
  }

  pub fn lint<N: VisitWith<CollectIdents>>(&self, expr: N) -> HashSet<Atom> {
    let mut collector = CollectIdents {
      idents: HashSet::new(),
      defined: HashSet::new(),
    };
    expr.visit_with(&mut collector);
    let found = collector.idents;
    found.difference(&self.declared).cloned().collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_lint_not_actually_variables() {
    let dts = vec![];

    let lint = LintUndefinedBindings::new(dts)
      .unwrap()
      .with_well_known_symbols();

    let src = r#"
    jsx(bar, { children: [jsx(Fragment, {})] });
    Object.defineProperties({}, {});
    { let y = 1; };
    let z = x + y;
    let x = Math.floor(Number("42.5"));
    let {a, b} = [1, 2, c], d;
    "#;

    let expr = parse_one(&src, None, parse_file_as_module).unwrap();

    let found = lint.lint(expr);

    assert_eq!(
      found,
      HashSet::from_iter(vec![
        "jsx".into(),
        "bar".into(),
        "Fragment".into(),
        // variables defined in the script are ignored to the best of our effort
        // in particular, there would be no scoping analysis, etc.
        // otherwise this would become a full-blown linter.
        //
        // in the test case:
        "x".into(),
        // declared except after it's used
        // "y".into(), // false negative: declared but its scope ignored
        // "z".into(), // declared
        "c".into(),
        // not declared
        "d".into(),
        // declared but not initialized
      ])
    );
  }
}
