use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
  parse::{Nothing, Parser},
  parse_macro_input,
  spanned::Spanned,
  ItemStruct,
};

#[proc_macro_attribute]
pub fn basic_attributes(
  args: proc_macro::TokenStream,
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let _ = parse_macro_input!(args as Nothing);
  let mut ast = parse_macro_input!(input as ItemStruct);

  match ast.fields {
    syn::Fields::Named(ref mut fields) => {
      for attr in ["ids", "classes", "names", "dupnames"] {
        let name = Ident::new(attr, Span::call_site());
        fields.named.push(
          syn::Field::parse_named
            .parse2(quote! { #name: std::vec::Vec<std::string::String> })
            .unwrap(),
        )
      }
    }
    _ => {
      return syn::Error::new(ast.span(), "expected a struct with named fields")
        .to_compile_error()
        .into()
    }
  }

  quote! { #ast }.into()
}
