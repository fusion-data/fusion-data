use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{AngleBracketedGenericArguments, Attribute, GenericArgument, PathArguments, Type, TypePath};

fn inject_error_tip() -> syn::Error {
  syn::Error::new(Span::call_site(), "inject Service only support Named-field Struct")
}

enum InjectableType {
  Component(syn::Path),
  Config(syn::Path),
  ComponentRef(syn::Path),
  ConfigRef(syn::Path),
}

struct Injectable {
  ty: InjectableType,
  field_name: syn::Ident,
}

impl Injectable {
  fn new(field: syn::Field) -> syn::Result<Self> {
    let syn::Field { ident, ty, attrs, .. } = field;
    let type_path = if let syn::Type::Path(path) = ty { path.path } else { Err(inject_error_tip())? };
    Ok(Self { ty: Self::compute_type(attrs, type_path)?, field_name: ident.ok_or_else(inject_error_tip)? })
  }

  fn compute_type(attrs: Vec<Attribute>, ty: syn::Path) -> syn::Result<InjectableType> {
    for attr in attrs {
      if attr.path().is_ident("config") {
        return Ok(InjectableType::Config(ty));
      }
      if attr.path().is_ident("component") {
        return Ok(InjectableType::Component(ty));
      }
    }
    let last_path_segment = ty.segments.last().ok_or_else(inject_error_tip)?;
    if last_path_segment.ident == "ComponentRef" {
      return Ok(InjectableType::ComponentRef(Self::get_argument_type(&last_path_segment.arguments)?));
    }
    if last_path_segment.ident == "ConfigRef" {
      return Ok(InjectableType::ConfigRef(Self::get_argument_type(&last_path_segment.arguments)?));
    }
    eprintln!("type path: {:?}, {:#?}", ty, last_path_segment);
    Ok(InjectableType::Component(ty))
  }

  fn get_argument_type(path_args: &PathArguments) -> syn::Result<syn::Path> {
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = path_args {
      let ty = args.last().ok_or_else(inject_error_tip)?;
      if let GenericArgument::Type(Type::Path(TypePath { path, .. })) = ty {
        return Ok(path.clone());
      }
    }
    Err(inject_error_tip())
  }
}

impl ToTokens for Injectable {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Self { ty, field_name } = self;
    match ty {
      InjectableType::Component(type_path) => {
        tokens.extend(quote! {
            #field_name: app.get_component::<#type_path>().expect("")
        });
      }
      InjectableType::Config(type_path) => {
        tokens.extend(quote! {
            #field_name: app.get_config::<#type_path>()?
        });
      }
      InjectableType::ComponentRef(type_path) => {
        tokens.extend(quote! {
            #field_name: app.get_component_ref::<#type_path>().expect("")
        });
      }
      InjectableType::ConfigRef(type_path) => {
        tokens.extend(quote! {
            #field_name: ::ultimate::config::ConfigRef::new(app.get_config::<#type_path>()?)
        });
      }
    }
  }
}

struct Component {
  fields: Vec<Injectable>,
}

impl Component {
  fn new(fields: syn::Fields) -> syn::Result<Self> {
    let fields = fields.into_iter().map(Injectable::new).collect::<syn::Result<Vec<_>>>()?;
    Ok(Self { fields })
  }
}

impl ToTokens for Component {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let fields = &self.fields;
    tokens.extend(quote! {
        Self {
            #(#fields),*
        }
    });
  }
}

pub(crate) fn expand_derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
  let service = if let syn::Data::Struct(data) = input.data {
    Component::new(data.fields)?
  } else {
    return Err(inject_error_tip());
  };
  let ident = input.ident;
  let service_registrar = syn::Ident::new(&format!("__ComponentRegistrarFor_{ident}"), ident.span());

  let output = quote! {
    impl ::ultimate::component::Component for #ident {
      fn build(app: &::ultimate::application::ApplicationBuilder) -> ::ultimate::Result<Self> {
        use ::ultimate::configuration::ConfigRegistry;
        Ok(#service)
      }
    }

    #[allow(non_camel_case_types)]
    struct #service_registrar;

    impl ::ultimate::component::ComponentRegistrar for #service_registrar{
      fn install_component(&self, app: &mut ::ultimate::application::ApplicationBuilder)->::ultimate::Result<()> {
        app.add_component(#ident::build(app)?);
        Ok(())
      }
    }
    ::ultimate::submit_component!(#service_registrar);
  };

  Ok(output)
}
