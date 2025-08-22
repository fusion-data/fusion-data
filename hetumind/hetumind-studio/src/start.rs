use ultimate_core::{DataError, application::Application};
use ultimate_db::DbPlugin;

use crate::{
  endpoint::init_web,
  infra::{db::execution::ExecutionStorePlugin, queue::QueueProviderPlugin},
  runtime::workflow::WorkflowEnginePlugin,
  utils::NodeRegistryPlugin,
};

pub async fn start() -> Result<(), DataError> {
  // 初始化应用
  let app = Application::builder()
    .add_plugin(DbPlugin) // ModelManager
    .add_plugin(NodeRegistryPlugin) // NodeRegistry
    .add_plugin(ExecutionStorePlugin) // ExecutionStoreService
    .add_plugin(QueueProviderPlugin) // QueueProvider
    .add_plugin(WorkflowEnginePlugin) // WorkflowEngineService
    .run()
    .await?;

  // 初始化 web
  init_web(app).await?;

  Ok(())
}
