use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{ExecutionConfig, NodeRegistry};
use fusion_core::{application::ApplicationBuilder, configuration::ConfigRegistry, plugin::Plugin};

use crate::{
  infra::db::execution::{ExecutionStorePlugin, ExecutionStoreService},
  runtime::workflow::{DefaultWorkflowEngine, WorkflowEngineService},
  utils::NodeRegistryPlugin,
};

pub struct WorkflowEnginePlugin;

#[async_trait]
impl Plugin for WorkflowEnginePlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let execution_store: ExecutionStoreService = app.component();
    let node_registry: NodeRegistry = app.component();
    let config: ExecutionConfig = app.get_config_by_path("workflow.engine").unwrap();

    let workflow_engine: WorkflowEngineService =
      Arc::new(DefaultWorkflowEngine::new(node_registry, execution_store, config));
    app.add_component(workflow_engine);
  }

  fn dependencies(&self) -> Vec<&str> {
    vec![std::any::type_name::<ExecutionStorePlugin>(), std::any::type_name::<NodeRegistryPlugin>()]
  }
}
