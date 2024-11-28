use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
  punctuated::Punctuated, AngleBracketedGenericArguments, Attribute, GenericArgument, PathArguments, Type, TypePath,
};

fn inject_error_tip() -> syn::Error {
  syn::Error::new(Span::call_site(), "inject Component only support Named-field Struct")
}

// #[derive(Debug)]
enum InjectableType {
  Component(syn::Path),
  Config(syn::Path),
  ComponentArc(syn::Path),
  ConfigArc(syn::Path),
  Default,
}

impl InjectableType {
  pub fn get_path(&self) -> syn::Path {
    match self {
      InjectableType::Component(p) => p.clone(),
      InjectableType::Config(p) => p.clone(),
      InjectableType::ComponentArc(p) => p.clone(),
      InjectableType::ConfigArc(p) => p.clone(),
      InjectableType::Default => build_default_path(),
    }
  }
}

fn build_default_path() -> syn::Path {
  let mut segments = Punctuated::new();

  // 构建 "Default" 路径段
  let default_segment = syn::PathSegment {
    ident: syn::Ident::new("Default", proc_macro2::Span::call_site()),
    arguments: syn::PathArguments::None,
  };
  segments.push(default_segment);

  // 构建 "default" 路径段，这里它是 "Default" 的一个关联函数
  let default_fn_segment = syn::PathSegment {
    ident: syn::Ident::new("default", proc_macro2::Span::call_site()),
    arguments: syn::PathArguments::None,
  };
  segments.push(default_fn_segment);

  syn::Path { leading_colon: None, segments }
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
    if last_path_segment.ident == "ComponentArc" {
      return Ok(InjectableType::ComponentArc(Self::get_argument_type(&last_path_segment.arguments)?));
    }
    if last_path_segment.ident == "ConfigArc" {
      return Ok(InjectableType::ConfigArc(Self::get_argument_type(&last_path_segment.arguments)?));
    }

    // Ok(InjectableType::Component(ty))
    // XXX 非 config、component、ComponentArc、ConfigArc 类型，使用默认的 Default::default() 初始化
    eprintln!("[Missing] type path: {:?}, {:#?}", ty.to_token_stream(), last_path_segment.to_token_stream());
    Ok(InjectableType::Default)
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
      InjectableType::Component(type_path) => tokens.extend(quote! {
        #field_name: app.component::<#type_path>()
      }),
      InjectableType::Config(type_path) => tokens.extend(quote! {
        #field_name: app.get_config::<#type_path>()?
      }),
      InjectableType::ComponentArc(type_path) => tokens.extend(quote! {
        #field_name: match app.get_component_ref::<#type_path>() {
          Some(c) => c,
          None => panic!("ComponentArc not found, field_name: {}, type_path: {}", stringify!(#field_name), stringify!(#type_path)),
        }
      }),
      InjectableType::ConfigArc(type_path) => tokens.extend(quote! {
        #field_name: ::ultimate::config::ConfigArc::new(app.get_config::<#type_path>()?)
      }),
      InjectableType::Default => tokens.extend(quote! {
        #field_name: Default::default()
      }),
    }
  }
}

struct ComponentToTokens {
  fields: Vec<Injectable>,
}

impl ComponentToTokens {
  fn new(fields: syn::Fields) -> syn::Result<Self> {
    let fields = fields.into_iter().map(Injectable::new).collect::<syn::Result<Vec<_>>>()?;
    Ok(Self { fields })
  }
}

impl ToTokens for ComponentToTokens {
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
    ComponentToTokens::new(data.fields)?
  } else {
    return Err(inject_error_tip());
  };
  let ident = input.ident;
  let component_registrar = syn::Ident::new(&format!("__ComponentRegistrarFor_{ident}"), ident.span());
  // let static_component_registrar =
  // syn::Ident::new(&format!("__COMPONENT_REGISTRAR_DEPENDENCIES_FOR_{ident}"), ident.span());

  let dependencies: Vec<_> = component
    .fields
    .iter()
    .filter(|f| match f.ty {
      InjectableType::Component(_) => true,
      InjectableType::Config(_) => false,
      InjectableType::ComponentArc(_) => true,
      InjectableType::ConfigArc(_) => false,
      InjectableType::Default => false,
    })
    .map(|field| field.ty.get_path())
    .collect();
  // println!("\nComponent Name: {}, dependencies: {:?}", ident, dependencies);

  let token_stream = quote! {
    impl ::ultimate::component::Component for #ident {
      fn build(app: &::ultimate::application::ApplicationBuilder) -> ::ultimate::Result<Self> {
        use ::ultimate::configuration::ConfigRegistry;
        Ok(#component)
      }
    }

    // #[allow(non_upper_case_globals)]
    // static #static_component_registrar: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
    //   vec![#(std::any::type_name::<#dependencies>()),*]
    // });

    #[allow(non_camel_case_types)]
    struct #component_registrar;

    impl ::ultimate::component::ComponentInstaller for #component_registrar {
      fn dependencies(&self) -> Vec<&str> {
        vec![#(std::any::type_name::<#dependencies>()),*]
        // static_component_registrar.to_vec()
      }

      fn install_component(&self, app: &mut ::ultimate::application::ApplicationBuilder)->::ultimate::Result<()> {
        use ::ultimate::component::Component;
        app.add_component(#ident::build(app).unwrap());
        Ok(())
      }
    }
    ::ultimate::submit_component!(#component_registrar);
  };

  let output = token_stream;
  Ok(output)
}

#[allow(unused)]
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
