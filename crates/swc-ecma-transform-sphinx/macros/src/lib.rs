use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
  parse::Parser, parse_macro_input, spanned::Spanned, Field, Fields, Item, ItemEnum, ItemStruct,
};

#[proc_macro_attribute]
pub fn basic_attributes(
  args: proc_macro::TokenStream,
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let attributes = parse_macro_input!(args as proc_macro2::TokenStream);
  let mut input = parse_macro_input!(input as Item);
  match match input {
    Item::Struct(ref mut ast) => basic_attributes_for_struct(ast, &attributes),
    Item::Enum(ref mut ast) => basic_attributes_for_enum(ast, &attributes),
    _ => Err(syn::Error::new(input.span(), "expected a struct or enum")),
  } {
    Ok(()) => quote! { #input }.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

fn basic_attributes_for_struct(
  ast: &mut ItemStruct,
  attrs: &proc_macro2::TokenStream,
) -> syn::Result<()> {
  update_fields(&mut ast.fields, attrs)
}

fn basic_attributes_for_enum(
  ast: &mut ItemEnum,
  attrs: &proc_macro2::TokenStream,
) -> syn::Result<()> {
  ast
    .variants
    .iter_mut()
    .try_for_each(|variant| match variant.fields {
      syn::Fields::Unnamed(_) => Ok(()),
      _ => update_fields(&mut variant.fields, attrs),
    })?;
  Ok(())
}

fn update_fields(fields: &mut Fields, attrs: &proc_macro2::TokenStream) -> syn::Result<()> {
  if let syn::Fields::Unit = fields {
    *fields = syn::Fields::Named(syn::FieldsNamed {
      brace_token: Default::default(),
      named: Default::default(),
    });
  }

  match fields {
    syn::Fields::Named(ref mut fields) => {
      for attr in ["ids", "classes", "names", "dupnames"] {
        let name = Ident::new(attr, Span::call_site());
        fields.named.push(Field::parse_named.parse2(quote! {
          #attrs
          #name: std::vec::Vec<std::string::String>
        })?);
      }
      Ok(())
    }
    syn::Fields::Unit => unreachable!(),
    syn::Fields::Unnamed(fields) => Err(syn::Error::new(
      fields.span(),
      "expected a struct or enum variant with named fields",
    )),
  }
}
