use fusion_core::{
  DataError,
  application::{Application, ApplicationBuilder},
};
use fusion_db::DbPlugin;

use crate::{
  endpoint::init_web,
  infra::{
    binary_storage::BinaryDataManagerPlugin, db::execution::ExecutionStorePlugin, queue::QueueProviderPlugin,
    security::EncryptionKeyManager,
  },
  runtime::workflow::WorkflowEnginePlugin,
  utils::NodeRegistryPlugin,
};

pub fn app_builder<T>(extra_source: Option<T>) -> ApplicationBuilder
where
  T: config::Source + Send + Sync + 'static,
{
  let mut app = Application::builder();

  if let Some(source) = extra_source {
    app.add_config_source(source);
  }
  app.add_plugin(DbPlugin) // ModelManager
    .add_plugin(NodeRegistryPlugin) // NodeRegistry
    .add_plugin(ExecutionStorePlugin) // ExecutionStoreService
    .add_plugin(QueueProviderPlugin) // QueueProvider
    .add_plugin(BinaryDataManagerPlugin) // BinaryDataManager
    .add_plugin(WorkflowEnginePlugin); // WorkflowEngineService

  app.add_component(EncryptionKeyManager::new());

  app
}

pub async fn start() -> Result<(), DataError> {
  let app = app_builder::<config::Config>(None).run().await?;

  // 初始化 web
  init_web(app).await
}
