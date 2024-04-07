use swc_core::ecma::ast::CallExpr;

mod ast;
mod builder;
mod runtime;

pub mod fixes;
pub mod tag;
pub mod unpack;

use self::ast::JSXCall;
pub use self::{
  ast::{JSXElement, JSXElementMut},
  builder::{create_element, create_fragment, jsx_builder2, DocumentBuilder, JSXDocument},
  runtime::{JSXRuntime, JSXRuntimeDefault},
};

#[inline(always)]
pub fn jsx<R: JSXRuntime>(call: &CallExpr) -> Option<&(impl JSXElement<R> + '_)> {
  match <CallExpr as JSXCall<R>>::is_jsx(call) {
    Some(_) => Some(call),
    None => None,
  }
}

#[inline(always)]
pub fn jsx_mut<R: JSXRuntime>(call: &mut CallExpr) -> Option<&mut (impl JSXElementMut<R> + '_)> {
  match <CallExpr as JSXCall<R>>::is_jsx(call) {
    Some(_) => Some(call),
    None => None,
  }
}
