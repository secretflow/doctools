use std::vec;

use convert_case::{Case, Casing as _};
use quote::quote;
use syn::{spanned::Spanned as _, Expr, ExprAssign, ExprPath, ItemStruct};

#[proc_macro_derive(ESFunction, attributes(deno))]
pub fn derive_es_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item = syn::parse_macro_input!(input as syn::ItemStruct);
  match syn_es_function(item) {
    Ok(expanded) => expanded,
    Err(err) => err.to_compile_error(),
  }
  .into()
}

fn syn_es_function(item: syn::ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
  let name = &item.ident;

  let export_name = syn_export_name(&item)?
    .or_else(|| {
      let name = name.to_string().to_case(Case::Camel);
      Some(quote! { #name })
    })
    .unwrap();

  let to_args = syn_to_args(&item)?;

  let generics = &item.generics;

  let fn_body = quote! {
    #[automatically_derived]
    impl #generics deno_lite::ESFunction for #name #generics {
      fn export_name() -> &'static str {
        #export_name
      }

      fn to_args<'a>(
        &'a self,
        scope: &mut deno_lite::v8::HandleScope<'a>,
      ) -> deno_lite::serde_v8::Result<Vec<deno_lite::v8::Local<'a, deno_lite::v8::Value>>> {
        #to_args
      }
    }
  };

  Ok(fn_body.into())
}

fn syn_export_name(input: &ItemStruct) -> syn::Result<Option<proc_macro2::TokenStream>> {
  for attr in &input.attrs[..] {
    if !attr.path().is_ident("deno") {
      continue;
    }
    let expr: Expr = attr.parse_args()?;
    match expr {
      Expr::Assign(ExprAssign { left, right, .. }) => match *left {
        Expr::Path(ExprPath { path: left, .. }) if left.is_ident("export") => match *right {
          Expr::Path(ExprPath { path: right, .. }) => {
            let right = right.require_ident()?.to_string();
            return Ok(Some(quote! { #right }));
          }
          _ => return Err(syn::Error::new(right.span(), "expected identifier")),
        },
        _ => {}
      },
      _ => {}
    }
  }
  Ok(None)
}

fn syn_to_args(input: &ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
  let mut args = vec![];

  match input.fields {
    syn::Fields::Unit => {}
    syn::Fields::Named(_) => args.push(quote! {
      deno_lite::serde_v8::to_v8(scope, &self)?
    }),
    syn::Fields::Unnamed(ref fields) => {
      for (i, _) in fields.unnamed.iter().enumerate() {
        let i = syn::Index::from(i);
        args.push(quote! {
          deno_lite::serde_v8::to_v8(scope, self.#i)?
        });
      }
    }
  }

  let fn_body = quote! {
    let mut args = vec![
      #(#args),*
    ];
    std::result::Result::Ok(args)
  };

  Ok(fn_body.into())
}
