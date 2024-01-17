use swc_core::{
    common::util::take::Take as _,
    ecma::{
        ast::{ArrayLit, CallExpr, Expr, ExprOrSpread, KeyValueProp, Lit, ObjectLit, Str},
        visit::{
            noop_visit_mut_type, noop_visit_type, Visit, VisitMut, VisitMutWith as _,
            VisitWith as _,
        },
    },
};
use swc_utils::jsx::factory::JSXFactory;

use crate::message::{
    is_empty_or_whitespace,
    jsx::{JSXMessage, Palpable},
    Message,
};

/// For [phrasing][ContentModel::Phrasing] content, transform is done in two phases.
///
/// 1. [PhrasingContentPreflight] visits the tree **immutably** and determines if the element
///    is translatable i.e. if any non-whitespace text is present within the element
///    (think [Element.innerText])
/// 2. If it is indeed translatable, [PhrasingContentCollector] visits the tree **mutably**
///    and transform it into `<Trans>`
///
/// [Element.innerText]: https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/innerText
///
/// The first visit obviously adds extra overhead, but the alternative would be trying
/// to determine whether the element is translatable while borrowing it mutably. Because
/// whether the element has any text cannot be readily determined without visiting its
/// (arbitrarily deep) descendants, trying to avoid `mut` until proven necessary would
/// involve a lot of backtracking / conditionals / very fragile
/// [AST node taking][swc_core::common::util::take::Take]. This is much less ergonomic and
/// more error-prone than just visiting the tree twice.
pub struct PhrasingContentPreflight {
    is_translatable: bool,
}

impl Visit for PhrasingContentPreflight {
    noop_visit_type!();

    fn visit_call_expr(&mut self, call: &CallExpr) {
        if self.is_translatable {
            return;
        }
        call.visit_children_with(self);
    }

    fn visit_key_value_prop(&mut self, prop: &KeyValueProp) {
        if !JSXFactory::prop_is_children(prop) {
            return;
        }

        self.is_translatable = match &*prop.value {
            Expr::Array(ArrayLit { ref elems, .. }) => elems.iter().any(|expr| match expr {
                Some(ExprOrSpread { expr, .. }) => match &**expr {
                    Expr::Lit(Lit::Str(Str { value, .. })) => !is_empty_or_whitespace(&value),
                    _ => false,
                },
                None => false,
            }),
            Expr::Lit(Lit::Str(text)) => !is_empty_or_whitespace(&text.value),
            _ => false,
        };

        if !self.is_translatable {
            prop.visit_children_with(self);
        }
    }
}

impl PhrasingContentPreflight {
    pub fn new() -> Self {
        Self {
            is_translatable: false,
        }
    }

    pub fn is_translatable(&self) -> bool {
        self.is_translatable
    }
}

#[derive(Debug)]
pub struct PhrasingContentCollector {
    factory: JSXFactory,
    trans: String,
    message: JSXMessage,
}

impl VisitMut for PhrasingContentCollector {
    noop_visit_mut_type!();

    fn visit_mut_key_value_prop(&mut self, prop: &mut KeyValueProp) {
        if !JSXFactory::prop_is_children(prop) {
            return;
        }

        let children = match *prop.value.take() {
            Expr::Array(ArrayLit { mut elems, .. }) => elems
                .iter_mut()
                .filter_map(|expr| match expr {
                    None => None,
                    Some(ExprOrSpread { expr, .. }) => Some(expr.take()),
                })
                .collect::<Vec<_>>(),
            expr => vec![Box::from(expr)],
        };

        children
            .into_iter()
            .for_each(|mut expr| match *expr.take() {
                Expr::Lit(Lit::Str(lit)) => match self.message.text(&lit.value) {
                    Palpable(true) => (),
                    Palpable(false) => (),
                },
                Expr::Call(mut call) => match self.factory.call_is_jsx(&call) {
                    Some(_) => {
                        let idx = self.message.enter();
                        call.visit_mut_children_with(self);
                        self.message.exit(idx, Box::from(call.take()));
                    }
                    None => {
                        self.message.interpolate(Box::from(call.take()));
                    }
                },
                expr => {
                    self.message.interpolate(Box::from(expr));
                }
            });
    }

    fn visit_mut_object_lit(&mut self, object: &mut ObjectLit) {
        object.visit_mut_children_with(self);
        object.props = object
            .props
            .drain(..)
            .filter(|prop| {
                prop.as_prop()
                    .and_then(|p| p.as_key_value())
                    .and_then(|p| Some(!p.value.is_invalid()))
                    .unwrap_or(false)
            })
            .collect();
    }
}

impl PhrasingContentCollector {
    pub fn new(factory: JSXFactory, trans: &str, pre: bool) -> Self {
        Self {
            factory,
            trans: String::from(trans),
            message: JSXMessage::new(pre),
        }
    }

    pub fn result(self) -> (Message, Box<Expr>) {
        self.message.make_trans(&self.factory, &self.trans)
    }
}
