use async_trait::async_trait;
use hetumind_core::workflow::{NodeRegistry, RegistrationError};
use hetumind_nodes::{core, integration, trigger};
use fusion_core::{application::ApplicationBuilder, plugin::Plugin};

/// 初始化节点注册表
fn init_node_registry() -> Result<NodeRegistry, RegistrationError> {
  let node_registry = NodeRegistry::new();

  core::register_nodes(&node_registry)?;
  trigger::register_nodes(&node_registry)?;
  integration::register_nodes(&node_registry)?;

  Ok(node_registry)
}

pub struct NodeRegistryPlugin;

#[async_trait]
impl Plugin for NodeRegistryPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let node_registry = init_node_registry().unwrap();
    app.add_component(node_registry);
  }
}
