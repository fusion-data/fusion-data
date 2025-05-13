#![doc = include_str!("../../DI.md")]
mod error;

pub use inventory::submit;
use std::{any::Any, collections::HashSet, ops::Deref, sync::Arc};
use tracing::debug;
pub use ultimate_core_macros::Component;

use crate::application::ApplicationBuilder;

pub use error::{ComponentError, ComponentResult};

/// Component's dyn trait reference
#[derive(Debug, Clone)]
pub struct DynComponentArc(Arc<dyn Any + Send + Sync>);

impl DynComponentArc {
  /// constructor
  pub fn new<T>(component: T) -> Self
  where
    T: Any + Send + Sync,
  {
    Self(Arc::new(component))
  }

  /// Downcast to the specified type
  pub fn downcast<T>(self) -> ComponentResult<ComponentArc<T>>
  where
    T: Any + Send + Sync,
  {
    match self.0.downcast::<T>() {
      Ok(item) => Ok(ComponentArc::new(item)),
      Err(_) => Err(ComponentError::ComponentTypeMismatch(std::any::type_name::<T>())),
    }
  }
}

/// A component reference of a specified type
#[derive(Debug, Clone)]
pub struct ComponentArc<T>(Arc<T>);

impl<T> ComponentArc<T> {
  fn new(target_ref: Arc<T>) -> Self {
    Self(target_ref)
  }

  /// Get the raw pointer of the component
  #[inline]
  pub fn into_raw(self) -> *const T {
    Arc::into_raw(self.0)
  }
}

impl<T> Deref for ComponentArc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub trait Component: Clone + Sized + 'static {
  /// Construct the Component
  fn build(app: &ApplicationBuilder) -> crate::Result<Self>;
}

pub trait ComponentInstaller: Send + Sync + 'static {
  /// Get the dependencies of the Component
  fn dependencies(&self) -> Vec<&str>;

  /// Install the Component into the Application
  fn install_component(&self, app: &mut ApplicationBuilder) -> crate::Result<()>;
}

inventory::collect!(&'static dyn ComponentInstaller);

/// auto_config
#[macro_export]
macro_rules! submit_component {
  ($ty:tt) => {
    ::ultimate_core::component::submit! {
      &($ty) as &dyn ::ultimate_core::component::ComponentInstaller
    }
  };
}

/// Find all ComponentInstaller and install them into the application
pub fn auto_inject_component(app: &mut ApplicationBuilder) -> crate::Result<()> {
  let mut registrars: Vec<(&&dyn ComponentInstaller, Vec<&str>)> =
    inventory::iter::<&dyn ComponentInstaller>.into_iter().map(|cr| (cr, cr.dependencies())).collect();

  // TODO 当存在未注册的组件时，限制循环次数
  let mut epoch = 0;
  let mut last_unregister_len = 0;

  while !registrars.is_empty() && epoch < 10 {
    let mut unregistrars = vec![];
    for (registrar, deps) in registrars {
      let deps: Vec<&str> = deps.into_iter().filter(|d| !app.components.contains_key(*d)).collect();
      if deps.is_empty() {
        registrar.install_component(app)?;
      } else {
        debug!("Dependency does not exist, waiting for the next round: [{:?}]", deps);
        unregistrars.push((registrar, deps));
      }
    }
    if last_unregister_len == unregistrars.len() {
      epoch += 1;
    } else {
      epoch = 0;
      last_unregister_len = unregistrars.len();
    }
    registrars = unregistrars;
  }
  if epoch != 0 {
    let deps: HashSet<&str> = registrars.iter().flat_map(|(_, deps)| deps).copied().collect();
    panic!(
      "Component registration failed, please check the component dependency relationship. Unregistered Components: {:?}",
      deps
    );
  }
  Ok(())
}
