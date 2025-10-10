use fusion_core::{DataError, application::Application};
use fusion_db::DbPlugin;

use crate::{
  endpoint::init_web,
  infra::{db::execution::ExecutionStorePlugin, queue::QueueProviderPlugin, security::EncryptionKeyManager},
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

  app.add_component(EncryptionKeyManager::new());

  // 初始化 web
  init_web(app).await?;

  Ok(())
}
