use std::sync::OnceLock;

use ultimate::{application::ApplicationBuilder, component::ComponentInstaller};
use ultimate_common::time::UtcDateTime;

pub struct ComponentRegistrarWrapper(pub Box<dyn ComponentInstaller>);

inventory::collect!(ComponentRegistrarWrapper);

pub struct ExampleRegistrar;

// static DD: [&str; 1] = [std::any::type_name::<UtcDateTime>()];

static __EXAMPLE_REGISTRAR: OnceLock<Vec<&'static str>> = OnceLock::new();

impl ComponentInstaller for ExampleRegistrar {
  fn dependencies(&self) -> Vec<&str> {
    // self.dependencies.iter().map(AsRef::as_ref).collect()
    __EXAMPLE_REGISTRAR.get_or_init(|| vec![std::any::type_name::<UtcDateTime>()]).to_vec()
  }

  fn install_component(&self, _app: &mut ApplicationBuilder) -> ultimate::Result<()> {
    todo!()
  }
}

ultimate::component::submit!(&ExampleRegistrar as &dyn ComponentInstaller);

fn main() {}
