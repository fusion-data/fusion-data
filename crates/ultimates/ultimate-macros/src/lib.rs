use proc_macro::TokenStream;
use syn::DeriveInput;

mod builder;
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

/// Builder
#[proc_macro_derive(Builder)]
pub fn derive_builder(item: TokenStream) -> TokenStream {
  builder::create_builder(item.into()).into()
}
