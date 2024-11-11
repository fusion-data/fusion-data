pub use inventory::submit;
use std::{any::Any, ops::Deref, sync::Arc};
pub use ultimate_macros::Component;

use crate::application::ApplicationBuilder;

/// Component's dyn trait reference
#[derive(Debug, Clone)]
pub struct DynComponentRef(Arc<dyn Any + Send + Sync>);

impl DynComponentRef {
  /// constructor
  pub fn new<T>(component: T) -> Self
  where
    T: Any + Send + Sync,
  {
    Self(Arc::new(component))
  }

  /// Downcast to the specified type
  pub fn downcast<T>(self) -> Option<ComponentRef<T>>
  where
    T: Any + Send + Sync,
  {
    self.0.downcast::<T>().ok().map(ComponentRef::new)
  }
}

/// A component reference of a specified type
#[derive(Debug, Clone)]
pub struct ComponentRef<T>(Arc<T>);

impl<T> ComponentRef<T> {
  fn new(target_ref: Arc<T>) -> Self {
    Self(target_ref)
  }

  /// Get the raw pointer of the component
  #[inline]
  pub fn into_raw(self) -> *const T {
    Arc::into_raw(self.0)
  }
}

impl<T> Deref for ComponentRef<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub trait Component: Clone + Sized {
  /// Construct the Component
  fn build(app: &ApplicationBuilder) -> crate::Result<Self>;
}

pub trait ComponentRegistrar: Send + Sync + 'static {
  /// Install the Component into the Application
  fn install_component(&self, app: &mut ApplicationBuilder) -> crate::Result<()>;
}

inventory::collect!(&'static dyn ComponentRegistrar);

/// auto_config
#[macro_export]
macro_rules! submit_component {
  ($ty:ident) => {
    ::ultimate::component::submit! {
      &$ty as &dyn ::ultimate::component::ComponentRegistrar
    }
  };
}

/// Find all ComponentRegistrar and install them into the application
pub fn auto_inject_component(app: &mut ApplicationBuilder) -> crate::Result<()> {
  for registrar in inventory::iter::<&dyn ComponentRegistrar> {
    registrar.install_component(app)?;
  }
  Ok(())
}
