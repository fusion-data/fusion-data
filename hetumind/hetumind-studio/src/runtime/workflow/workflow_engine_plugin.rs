use std::sync::Arc;

use async_trait::async_trait;
use fusion_core::{application::ApplicationBuilder, configuration::ConfigRegistry, plugin::Plugin};
use hetumind_core::workflow::{NodeRegistry, WorkflowEngineSetting};

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
    let setting: WorkflowEngineSetting = app.get_config_by_path("hetumind.workflow.engine").unwrap();

    let workflow_engine: WorkflowEngineService =
      Arc::new(DefaultWorkflowEngine::new(node_registry, execution_store, setting));
    app.add_component(workflow_engine);
  }

  fn dependencies(&self) -> Vec<&str> {
    vec![std::any::type_name::<ExecutionStorePlugin>(), std::any::type_name::<NodeRegistryPlugin>()]
  }
}
