// region:    --- Modules

mod derives_field;
mod derives_filter;
mod utils;

use crate::derives_filter::derive_filter_nodes_inner;
use proc_macro::TokenStream;

// endregion: --- Modules

#[proc_macro_derive(FilterNodes, attributes(fusionsql))]
pub fn derive_filter_nodes(input: TokenStream) -> TokenStream {
  derive_filter_nodes_inner(input)
}

// region:    --- with-sea-query

#[proc_macro_derive(Fields, attributes(field, fusionsql))]
pub fn derive_fields(input: TokenStream) -> TokenStream {
  derives_field::derive_fields_inner(input)
}

/// Implements `From<T> for sea_query::Value` and `sea_query::Nullable for T`
/// where T is the struct or enum annotated with `#[derive(Field)]` for simple
/// tuple structs or enums.
///
/// For more complex types, implement both of these traits for the type.
///
/// For example:
///
/// - On simple type and single element tuple struct
///
/// Notes:
///   - Supports only primitive types (no array yet)
///   - Supports only one tuple field.
///
/// Notes:
///   - Will be treated a sea_query::Value::String with the name of the variant.
///   - No rename for now.
#[proc_macro_derive(SeaFieldValue)]
pub fn derive_field_sea_value(input: TokenStream) -> TokenStream {
  derives_field::derive_field_sea_value_inner(input)
}

// endregion: --- with-sea-query
