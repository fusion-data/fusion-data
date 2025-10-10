use crate::utils::{get_field_attribute, get_meta_value_string};
use proc_macro2::Ident;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Field, FieldsNamed, Meta, Token};

// region:    --- Field Prop (i.e., sqlb Field)
pub struct ModelsqlFieldProp<'a> {
  pub prop_name: String,         // property name
  pub attr_name: Option<String>, // The eventual `#[field(name=..._)]`
  pub name: String,              // resolved name attr_name or prop name;
  pub rel: Option<String>,
  pub cast_as: Option<String>,
  pub is_option: bool,
  pub ident: &'a Option<Ident>,
}

// pub fn get_fusionsql_field_props(fields: &FieldsNamed) -> Vec<ModelsqlFieldProp> {
//   let fusionsql_fields_and_skips = get_fusionsql_field_props_and_skips(fields);
//   fusionsql_fields_and_skips.fusionsql_fields
// }

pub struct ModelsqlFieldsAndSkips<'a> {
  pub fusionsql_fields: Vec<ModelsqlFieldProp<'a>>,
  #[allow(unused)] // For early development.
  pub skipped_fields: Vec<&'a Field>,
  pub field_mask_field: Option<&'a Field>,
}

pub fn get_fusionsql_field_props_and_skips<'a>(fields: &'a FieldsNamed) -> ModelsqlFieldsAndSkips<'a> {
  let mut fusionsql_fields = Vec::new();
  let mut skipped_fields = Vec::new();
  let mut field_mask_field: Option<&Field> = None;

  for field in fields.named.iter() {
    // -- Get the FieldAttr
    let mfield_attr = get_mfield_prop_attr(field);

    // TODO: Need to check better handling.
    let mfield_attr = mfield_attr.unwrap();

    if mfield_attr.is_field_mask {
      field_mask_field = Some(field);
      continue;
    }
    if mfield_attr.skip {
      skipped_fields.push(field);
      continue;
    }

    // -- ident
    let ident = &field.ident;

    // -- is_option
    // NOTE: By macro limitation, we can do only type name match and it would not support type alias
    //       For now, assume Option is used as is or type name contains it.
    //       We can add other variants of Option if proven needed.
    let type_name = format!("{}", &field.ty.to_token_stream());
    let is_option = type_name.contains("Option ");

    // -- name
    let prop_name = ident.as_ref().map(|i| i.to_string()).unwrap();
    let attr_name = mfield_attr.name;
    let name = attr_name.clone().unwrap_or_else(|| prop_name.clone());

    // -- cast_as
    let cast_as = mfield_attr.cast_as;

    // -- Add to array.
    fusionsql_fields.push(ModelsqlFieldProp {
      rel: mfield_attr.rel,
      name,
      prop_name,
      attr_name,
      ident,
      cast_as,
      is_option,
    })
  }

  ModelsqlFieldsAndSkips { fusionsql_fields, skipped_fields, field_mask_field }
}

// endregion: --- Field Prop (i.e., sqlb Field)

// region:    --- Field Prop Attribute
struct ModelsqlFieldPropAttr {
  pub rel: Option<String>,
  pub name: Option<String>,
  pub skip: bool,
  pub cast_as: Option<String>,
  pub is_field_mask: bool,
}

// #[field(skip)]
// #[field(name = "new_name")]
fn get_mfield_prop_attr(field: &Field) -> Result<ModelsqlFieldPropAttr, syn::Error> {
  let attribute = get_field_attribute(field, "field");

  let mut skip = false;
  let mut rel: Option<String> = None;
  let mut column: Option<String> = None;
  let mut cast_as: Option<String> = None;
  let mut is_field_mask = false;

  if is_field_mask_type(&field.ty) {
    // println!("is_field_mask_type: {:?}", field);
    // skip = true;
    is_field_mask = true;
  } else if let Some(attribute) = attribute {
    let nested = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for meta in nested {
      match meta {
        // #[field(skip)]
        Meta::Path(path) if path.is_ident("skip") => {
          skip = true;
        }

        // #[field(name=value)]
        Meta::NameValue(nv) => {
          if nv.path.is_ident("rel") {
            rel = get_meta_value_string(nv);
          } else if nv.path.is_ident("name") {
            column = get_meta_value_string(nv);
          } else if nv.path.is_ident("cast_as") {
            cast_as = get_meta_value_string(nv);
          }
        }

        /* ... */
        _ => {
          return Err(syn::Error::new_spanned(
            meta,
            r#"
Unrecognized #[field...] attribute. Accepted attribute
#[field(skip)]
or
#[field(rel="table_name", name="some_col_name", cast_as="sea query cast as type")]
"#,
          ));
        }
      }
    }
  }

  Ok(ModelsqlFieldPropAttr { skip, rel, name: column, cast_as, is_field_mask })
}

// endregion: --- Field Prop Attribute

// 检查字段类型，如果是 FieldMask 或 Option<FieldMask>，自动设置 skip = true
fn is_field_mask_type(ty: &syn::Type) -> bool {
  match ty {
    syn::Type::Path(type_path) => is_field_mask_path(&type_path.path),
    _ => false,
  }
}

fn is_field_mask_path(path: &syn::Path) -> bool {
  // 检查路径的最后一个段是否是 FieldMask
  if let Some(last_segment) = path.segments.last() {
    if last_segment.ident == "FieldMask" {
      // 检查完整路径是否符合预期
      return is_valid_field_mask_path(path);
    } else if last_segment.ident == "Option" {
      // 检查 Option<T> 中的 T
      if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
        && let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
      {
        return is_field_mask_type(inner_type);
      }
    }
  }
  false
}

fn is_valid_field_mask_path(path: &syn::Path) -> bool {
  let segments: Vec<&syn::Ident> = path.segments.iter().map(|s| &s.ident).collect();

  match segments.as_slice() {
    // 直接引用: FieldMask (已通过 use 导入)
    [ident] if *ident == "FieldMask" => true,

    // 相对路径: field::FieldMask
    [field, field_mask] if *field == "field" && *field_mask == "FieldMask" => true,

    // 完整路径: fusionsql::field::FieldMask
    [fusionsql, field, field_mask] if *fusionsql == "fusionsql" && *field == "field" && *field_mask == "FieldMask" => {
      true
    }

    // crate 路径: crate::field::FieldMask
    [crate_kw, field, field_mask] if *crate_kw == "crate" && *field == "field" && *field_mask == "FieldMask" => true,

    // super 路径: super::FieldMask, super::field::FieldMask
    [super_kw, field_mask] if *super_kw == "super" && *field_mask == "FieldMask" => true,
    [super_kw, field, field_mask] if *super_kw == "super" && *field == "field" && *field_mask == "FieldMask" => true,

    // self 路径: self::FieldMask, self::field::FieldMask
    [self_kw, field_mask] if *self_kw == "self" && *field_mask == "FieldMask" => true,
    [self_kw, field, field_mask] if *self_kw == "self" && *field == "field" && *field_mask == "FieldMask" => true,

    _ => false,
  }
}
