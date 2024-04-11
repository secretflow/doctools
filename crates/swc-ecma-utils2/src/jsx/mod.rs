mod ast;
mod builder;
mod runtime;
mod tag;
mod unpack;

pub mod fixes;

pub use self::{
  ast::{JSXElement, JSXElementMut},
  builder::{
    create_element, create_fragment, jsx_builder2, replace_element, DocumentBuilder, JSXDocument,
  },
  runtime::{JSXRuntime, JSXSymbols},
  tag::{JSXTagDef, JSXTagType, JSXTagTypeOwned},
  unpack::{unpack_jsx, TextNode, UnpackJSXError},
};
