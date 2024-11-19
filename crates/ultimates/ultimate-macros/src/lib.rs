use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

mod configuration;
mod helpers;
mod inject;

/// Configuration
#[proc_macro_derive(Configuration, attributes(config_prefix))]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  configuration::expand_derive(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Injectable Component
#[proc_macro_derive(Component, attributes(config, component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);

  inject::expand_derive(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

#[proc_macro_derive(PrintTypePaths)]
pub fn print_type_paths(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let name = input.ident;
  let data = &input.data;
  let mut output = Vec::new();

  match data {
    Data::Struct(ref struct_data) => match struct_data.fields {
      Fields::Named(ref fields) => {
        for field in fields.named.iter() {
          let field_type = &field.ty;

          let type_path = quote! {
              println!("Type path of field {:?} is {:?}", stringify!(#field_type), std::any::type_name::<#field_type>());
          };
          output.push(type_path);
        }
      }
      Fields::Unnamed(ref fields) => {
        for (i, field) in fields.unnamed.iter().enumerate() {
          let field_type = &field.ty;
          let type_path = quote! {
              println!("Type path of field {} is {:?}", i, #field_type);
          };
          output.push(type_path);
        }
      }
      Fields::Unit => {}
    },
    Data::Enum(_) | Data::Union(_) => {
      // 对于枚举和联合体可以添加相应的处理逻辑
    }
  }

  let expanded = quote! {
      impl #name {
          pub fn print_type_paths() {
              #(#output)*
          }
      }
  };

  TokenStream::from(expanded)
}
