use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, DeriveInput, Fields, FieldsNamed};

pub fn create_builder(item: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse2(item).unwrap();
  let name = ast.ident;
  let builder = format_ident!("{}Builder", name);

  let fields = match ast.data {
    syn::Data::Struct(DataStruct { fields: Fields::Named(FieldsNamed { ref named, .. }), .. }) => named,
    // syn::Data::Enum(_) => todo!(),
    // syn::Data::Union(_) => todo!(),
    _ => unimplemented!("Only implemented for structs"),
  };

  let builder_fields = fields.iter().map(|f| {
    let field_name = &f.ident;
    let field_type = &f.ty;
    quote! { #field_name: Option<#field_type> }
  });
  let builder_hints = fields.iter().map(|f| {
    let field_name = &f.ident;
    quote! { #field_name: None }
  });
  let builder_methods = fields.iter().map(|f| {
    let field_name = &f.ident;
    let field_type = &f.ty;
    quote! {
      pub fn #field_name(&mut self, input: #field_type) -> &mut Self {
        self.#field_name = Some(input);
        self
      }
    }
  });
  let set_fields = fields.iter().map(|f| {
    let field_name = &f.ident;
    let field_name_as_string = field_name.as_ref().unwrap().to_string();
    quote! {
      #field_name: self.#field_name.as_ref().expect(&format!("field '{}' not set", #field_name_as_string)).clone()
    }
  });

  quote! {
    struct #builder {
      #(#builder_fields,)*
    }

    impl #builder {
      #(#builder_methods)*

      pub fn build(&self) -> #name {
        #name {
          #(#set_fields,)*
        }
      }
    }

    impl #name {
      pub fn builder() -> #builder {
        #builder {
          #(#builder_hints,)*
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn builder_struct_name_should_be_present_in_output() {
    let input = quote! {
      struct StructWithNoFields {}
    };

    let actual = create_builder(input);
    let actual_string = actual.to_string();

    // 检查生成的代码包含 builder 结构体
    assert!(actual_string.contains("struct StructWithNoFieldsBuilder"));
    assert!(actual_string.contains("impl StructWithNoFieldsBuilder"));
    assert!(actual_string.contains("impl StructWithNoFields"));
  }

  #[test]
  fn assert_with_parsing() {
    let input = quote! {
      struct StructWithNoFields {}
    };

    let actual = create_builder(input);

    // 生成的代码包含多个项目，不能直接解析为单个 DeriveInput
    // 而是应该解析为多个项目，检查第一个是否为 builder 结构体
    let parsed: syn::File = syn::parse2(quote! { #actual }).unwrap();

    // 获取第一个项目（应该是 builder 结构体）
    if let Some(syn::Item::Struct(struct_item)) = parsed.items.first() {
      assert_eq!(struct_item.ident.to_string(), "StructWithNoFieldsBuilder");
    } else {
      panic!("第一个项目应该是 builder 结构体");
    }
  }
}
