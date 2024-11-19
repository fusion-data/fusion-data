use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{AngleBracketedGenericArguments, Attribute, GenericArgument, PathArguments, Type, TypePath};

fn inject_error_tip() -> syn::Error {
  syn::Error::new(Span::call_site(), "inject Component only support Named-field Struct")
}

#[derive(Debug)]
enum InjectableType {
  Component(syn::Path),
  Config(syn::Path),
  ComponentRef(syn::Path),
  ConfigRef(syn::Path),
}

impl InjectableType {
  pub fn get_path(&self) -> syn::Path {
    match self {
      InjectableType::Component(p) => p.clone(),
      InjectableType::Config(p) => p.clone(),
      InjectableType::ComponentRef(p) => p.clone(),
      InjectableType::ConfigRef(p) => p.clone(),
    }
  }
}

struct Injectable {
  pub ty: InjectableType,
  pub field_name: syn::Ident,
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
    eprintln!("[INVALID] type path: {:?}, {:#?}", ty, last_path_segment);
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
            #field_name: app.component::<#type_path>()
        });
      }
      InjectableType::Config(type_path) => {
        tokens.extend(quote! {
            #field_name: app.get_config::<#type_path>()?
        });
      }
      InjectableType::ComponentRef(type_path) => {
        tokens.extend(quote! {
            #field_name: match app.get_component_ref::<#type_path>() {
                Some(c) => c,
                None => panic!("ComponentRef not found, field_name."),
            }
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
  let component = if let syn::Data::Struct(data) = input.data {
    Component::new(data.fields)?
  } else {
    return Err(inject_error_tip());
  };
  let ident = input.ident;
  let component_registrar = syn::Ident::new(&format!("__ComponentRegistrarFor_{ident}"), ident.span());

  let dependencies: Vec<_> = component
    .fields
    .iter()
    .map(|field| {
      let type_path = field.ty.get_path();
      // quote! {
      //   std::any::type_name<#type_path>()
      // }
      type_path
    })
    .collect();
  // println!("\nComponent Name: {}, dependencies: {:?}", ident, dependencies);

  let output = quote! {
    impl ::ultimate::component::Component for #ident {
      fn build(app: &::ultimate::application::ApplicationBuilder) -> ::ultimate::Result<Self> {
        use ::ultimate::configuration::ConfigRegistry;
        Ok(#component)
      }
    }

    #[allow(non_camel_case_types)]
    struct #component_registrar;

    impl ::ultimate::component::ComponentRegistrar for #component_registrar {
      fn dependencies(&self) -> Vec<&str> {
        vec![#(std::any::type_name::<#dependencies>()),*]
      }

      fn install_component(&self, app: &mut ::ultimate::application::ApplicationBuilder)->::ultimate::Result<()> {
        app.add_component(#ident::build(app).unwrap());
        Ok(())
      }
    }
    ::ultimate::submit_component!(#component_registrar);
  };

  Ok(output)
}

fn get_full_path(ty: &Type) -> Option<String> {
  if let Type::Path(type_path) = ty {
    let mut segments = type_path.path.segments.iter().map(|seg| seg.ident.to_string()).collect::<Vec<_>>();
    if let Some(first_segment) = segments.first() {
      if first_segment == "crate" {
        segments.remove(0); // Remove "crate" from the path
      }
    }
    Some(segments.join("::"))
  } else {
    None
  }
}
