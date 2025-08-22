use crate::utils::modql_field::ModelsqlFieldProp;
use crate::utils::struct_modql_attr::{StructModqlFieldProps, get_struct_modelsql_props};
use crate::utils::{get_struct_fields, modql_field};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{DeriveInput, Field, parse_macro_input};

pub(crate) fn derive_fields_inner(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let fields = get_struct_fields(&ast);

  let struct_name = &ast.ident;

  // -- Collect Elements
  // Properties for all fields (with potential additional info with #[field(...)])
  let modelsql_fields_and_skips = modql_field::get_modelsql_field_props_and_skips(fields);
  let field_props = modelsql_fields_and_skips.modelsql_fields; //modql_field::get_modelsql_field_props(fields);
  let field_mask_field = modelsql_fields_and_skips.field_mask_field;

  let struct_modelsql_prop = get_struct_modelsql_props(&ast).unwrap();

  // Will be "" if none (this if for the struct #[modelsql(table = ...)])

  let impl_has_fields = impl_has_fields(struct_name, &struct_modelsql_prop, &field_props);

  let impl_names_as_consts = if let Some(names_as_consts) = struct_modelsql_prop.names_as_consts.as_deref() {
    //
    impl_names_as_consts(struct_name, &field_props, names_as_consts)
  } else {
    quote! {}
  };

  let impl_sea_fields = impl_has_sea_fields(struct_name, &struct_modelsql_prop, &field_props, field_mask_field);

  let output = quote! {
    #impl_has_fields

    #impl_names_as_consts

    #impl_sea_fields
  };

  output.into()
}

fn impl_names_as_consts(
  struct_name: &Ident,
  field_props: &[ModelsqlFieldProp<'_>],
  prop_name_prefix: &str,
) -> proc_macro2::TokenStream {
  // If prefix not empty, amek sure it ends with `_`
  let prop_name_prefix = if !prop_name_prefix.is_empty() && !prop_name_prefix.ends_with('_') {
    format!("{prop_name_prefix}_")
  } else {
    prop_name_prefix.to_string()
  };

  let consts = field_props.iter().map(|field| {
    let prop_name = &field.prop_name;
    let const_name = format!("{}{}", prop_name_prefix, prop_name.to_uppercase());
    let const_name = Ident::new(&const_name, Span::call_site());

    let name = &field.name;
    quote! {
      pub const #const_name: &'static str = #name;
    }
  });

  quote! {
    impl #struct_name {
      #(#consts)*
    }
  }
}

fn impl_has_fields(
  struct_name: &Ident,
  struct_modql_prop: &StructModqlFieldProps,
  field_props: &[ModelsqlFieldProp<'_>],
) -> proc_macro2::TokenStream {
  let props_all_names: Vec<&String> = field_props.iter().map(|p| &p.name).collect();

  let struct_rel = struct_modql_prop.rel.as_ref();

  // -- Build FieldRef quotes
  let props_field_refs = field_props.iter().map(|field_prop| {
    let name = field_prop.name.to_string();
    let rel = field_prop.rel.as_ref().or(struct_rel);
    let rel = match rel {
      Some(rel) => quote! { Some(#rel)},
      None => quote! { None },
    };
    quote! {&modelsql::field::FieldRef{rel: #rel, name: #name}}
  });

  // -- Build the FieldMeta quotes
  let props_field_metas = field_props.iter().map(|field_prop| {
    // This below is resolved in the FieldMeta implemntation (same logic)
    // let name = field_prop.name.to_string();

    let prop_name = field_prop.prop_name.to_string();

    let attr_name = match field_prop.attr_name.as_ref() {
      Some(attr_name) => quote! { Some(#attr_name)},
      None => quote! { None },
    };

    let field_rel = field_prop.rel.as_ref();

    let is_struct_rel = match (struct_rel, field_rel) {
      (Some(_), None) => true,
      (Some(struct_rel), Some(field_rel)) => struct_rel == field_rel,
      _ => false,
    };

    let rel = field_prop.rel.as_ref().or(struct_rel);
    let rel = match rel {
      Some(rel) => quote! { Some(#rel)},
      None => quote! { None },
    };
    let cast_as = match &field_prop.cast_as {
      Some(cast_as) => quote! { Some(#cast_as)},
      None => quote! { None },
    };
    let is_option = field_prop.is_option;

    quote! {&modelsql::field::FieldMeta{
        rel: #rel,
        is_struct_rel: #is_struct_rel,
        prop_name: #prop_name,
        attr_name: #attr_name,
        cast_as: #cast_as,
        is_option: #is_option,
      }
    }
  });

  let output = quote! {

    impl modelsql::field::HasFields for #struct_name {

      fn field_names() -> &'static [&'static str] {
        &[#(
        #props_all_names,
        )*]
      }

      fn field_refs() -> &'static [&'static modelsql::field::FieldRef] {
        &[#(
        #props_field_refs,
        )*]
      }

      fn field_metas() -> &'static modelsql::field::FieldMetas {
        static METAS: &[&modelsql::field::FieldMeta] = &[#(
        #props_field_metas,
        )*];

        static METAS_HOLDER: modelsql::field::FieldMetas = modelsql::field::FieldMetas::new(METAS);

        &METAS_HOLDER
      }

    }
  };

  output
}

fn impl_has_sea_fields(
  struct_name: &Ident,
  struct_modql_prop: &StructModqlFieldProps,
  field_props: &[ModelsqlFieldProp<'_>],
  field_mask_field: Option<&Field>,
) -> proc_macro2::TokenStream {
  let prop_all_names: Vec<&String> = field_props.iter().map(|p| &p.name).collect();

  // this will repeat the struct table name for all fields.
  let prop_all_rels: Vec<String> = field_props
    .iter()
    .map(|p| {
      p.rel
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| struct_modql_prop.rel.as_ref().map(|s| s.to_string()).unwrap_or_default())
    })
    .collect();

  fn field_options_quote(mfield_prop: &ModelsqlFieldProp) -> proc_macro2::TokenStream {
    if let Some(cast_as) = &mfield_prop.cast_as {
      quote! { modelsql::field::FieldOptions { cast_as: Some(#cast_as.to_string()) } }
    } else {
      quote! { modelsql::field::FieldOptions { cast_as: None } }
    }
  }

  // -- all_fields() quotes!
  let all_fields_quotes = field_props.iter().map(|p| {
    let name = &p.name;
    let field_options_q = field_options_quote(p);
    let ident = p.ident;

    quote! {
      ff.push(
        modelsql::field::SeaField::new_with_options(modelsql::SIden(#name), self.#ident.into(), #field_options_q)
      );
    }
  });

  // -- The not_none_sea_fields quotes!
  let not_none_fields_quotes = field_props.iter().map(|p| {
    let name = &p.name;
    let field_options_q = field_options_quote(p);
    let ident = p.ident;

    if p.is_option {
      quote! {
        if let Some(val) = self.#ident {
          ff.push(
            modelsql::field::SeaField::new_with_options(modelsql::SIden(#name), val.into(), #field_options_q)
          );
        }
      }
    } else {
      quote! {
        ff.push(
          modelsql::field::SeaField::new_with_options(modelsql::SIden(#name), self.#ident.into(), #field_options_q)
        );
      }
    }
  });

  // -- The sea_fields_with_mask quotes!
  let field_mask_ident = field_mask_field.and_then(|f| f.ident.clone());
  let is_option_field_mask = field_mask_field.is_some_and(|f| match &f.ty {
    syn::Type::Path(type_path) => type_path.path.segments.last().is_some_and(|segment| segment.ident == "Option"),
    _ => false,
  });
  let sea_fields_with_mask_quotes = field_props.iter().map(|p| {
    let name = &p.name;
    let field_options_q = field_options_quote(p);
    let ident = p.ident;

    // 生成推送字段的表达式
    let create_field_expr = |value_expr: proc_macro2::TokenStream| {
      quote! {
        ff.push(
          modelsql::field::SeaField::new_with_options(modelsql::SIden(#name), #value_expr.into(), #field_options_q)
        );
      }
    };

    match (field_mask_ident.as_ref(), is_option_field_mask, p.is_option) {
      // 没有 field mask
      (None, _, true) => {
        let field_expr = create_field_expr(quote! { val });
        quote! {
          if let Some(val) = self.#ident {
            #field_expr
          }
        }
      }
      (None, _, false) => create_field_expr(quote! { self.#ident }),

      // 有 field mask，且是 Option<FieldMask>
      (Some(mask_ident), true, p_is_option) => {
        let field_expr_set_value = create_field_expr(quote! { val });
        let field_expr = create_field_expr(quote! { self.#ident });
        quote! {
          if self.#mask_ident.is_none() && #p_is_option {
            if let Some(val) = self.#ident {
              #field_expr_set_value
            }
          } else {
            if self.#mask_ident.as_ref().map_or(true, |mask| mask.hit(#name)) {
              #field_expr
            }
          }
        }
      }

      // 有 field mask，且不是 Option (直接是 FieldMask)
      (Some(mask_ident), false, _) => {
        let field_expr = create_field_expr(quote! { self.#ident });
        quote! {
          if self.#mask_ident.hit(#name) {
            #field_expr
          }
        }
      }
    }
  });

  // -- Compose the final code
  let output = quote! {

    impl modelsql::field::HasSeaFields for #struct_name {

      fn not_none_sea_fields(self) -> modelsql::field::SeaFields {
        let mut ff: Vec<modelsql::field::SeaField> = Vec::new();
        #(#not_none_fields_quotes)*
        modelsql::field::SeaFields::new(ff)
      }

      fn sea_fields_with_mask(self) -> modelsql::field::SeaFields {
        let mut ff: Vec<modelsql::field::SeaField> = Vec::new();
        #(#sea_fields_with_mask_quotes)*
        modelsql::field::SeaFields::new(ff)
      }

      fn all_sea_fields(self) -> modelsql::field::SeaFields {
        let mut ff: Vec<modelsql::field::SeaField> = Vec::new();
        #(#all_fields_quotes)*
        modelsql::field::SeaFields::new(ff)
      }

      fn sea_idens() -> Vec<sea_query::SeaRc<dyn sea_query::Iden>> {
        vec![#(
        sea_query::IntoIden::into_iden(modelsql::SIden(#prop_all_names)),
        )*]
      }

      fn sea_column_refs() -> Vec<sea_query::ColumnRef> {
        use sea_query::IntoIden;
        use sea_query::ColumnRef;
        use modelsql::SIden;

        let mut v = Vec::new();

        // NOTE: There's likely a more elegant solution, but this approach is semantically correct.
        #(
          let col_ref = if #prop_all_rels == "" {
            ColumnRef::Column(SIden(#prop_all_names).into_iden())
          } else {
            ColumnRef::TableColumn(
              SIden(#prop_all_rels).into_iden(),
              SIden(#prop_all_names).into_iden())
          };
          v.push(col_ref);
        )*
        v
      }

      fn sea_column_refs_with_rel(rel_iden: impl sea_query::IntoIden) -> Vec<sea_query::ColumnRef> {
        use sea_query::IntoIden;
        use sea_query::ColumnRef;
        use modelsql::SIden;

        let rel_iden = rel_iden.into_iden();

        let mut v = Vec::new();

        // NOTE: There's likely a more elegant solution, but this approach is semantically correct.
        #(
          let col_ref =
            ColumnRef::TableColumn(
              rel_iden.clone(),
              SIden(#prop_all_names).into_iden());

          v.push(col_ref);
        )*
        v
      }
    }
  };

  output
}
