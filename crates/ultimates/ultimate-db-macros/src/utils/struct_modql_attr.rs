use crate::utils::{get_dinput_attribute, get_meta_value_string};
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Meta, Token};

// region:    --- Struct Prop Attribute
pub struct StructModqlFieldProps {
  pub rel: Option<String>,
  pub names_as_consts: Option<String>,
}

pub fn get_struct_modql_props(dinput: &DeriveInput) -> Result<StructModqlFieldProps, syn::Error> {
  // FIXME: We should remove this, 'sqlb' should not be a thing anymore.
  let sqlb_attr = get_dinput_attribute(dinput, "modql");
  let mut rel = None;
  let mut names_as_consts = None;

  if let Some(attribute) = sqlb_attr {
    let nested = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for meta in nested {
      match meta {
        // #[modql(rel=value)]
        Meta::NameValue(nv) =>
        {
          #[allow(clippy::if_same_then_else)]
          if nv.path.is_ident("rel") {
            rel = get_meta_value_string(nv);
          } else if nv.path.is_ident("names_as_consts") {
            names_as_consts = get_meta_value_string(nv);
          }
        }

        Meta::Path(path) => {
          if let Some(path) = path.get_ident() {
            let path = path.to_string();
            if path == "names_as_consts" {
              names_as_consts = Some("".to_string())
            }
          }
        }

        /* ... */
        _ => {
          let msg = "unrecognized modql attribute value";
          return Err(syn::Error::new_spanned(meta, msg));
          // return Err(syn::Error::new_spanned(meta, "unrecognized field"));
        }
      }
    }
  }

  Ok(StructModqlFieldProps { rel, names_as_consts })
}

// endregion: --- Struct Prop Attribute
