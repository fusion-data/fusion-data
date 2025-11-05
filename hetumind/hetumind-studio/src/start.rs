use fusion_core::{
  DataError,
  application::{Application, ApplicationBuilder},
};
use fusion_db::DbPlugin;

use crate::{
  domain::user::UserSyncSvc,
  endpoint::init_web,
  infra::{
    binary_storage::BinaryDataManagerPlugin, db::execution::ExecutionStorePlugin, queue::QueueProviderPlugin,
    security::EncryptionKeyManager,
  },
  runtime::workflow::WorkflowEnginePlugin,
  utils::NodeRegistryPlugin,
};
use hetumind_context::services::memory_service::{InMemoryMemoryService, MemoryService};
use std::sync::Arc;

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
  // 注册默认 MemoryService 组件（InMemory + TTL 300s），供 SimpleMemorySupplier/EngineRouter 使用
  let memory_service: Arc<dyn MemoryService> = Arc::new(InMemoryMemoryService::new(300));
  app.add_component::<Arc<dyn MemoryService>>(memory_service);

  app
}

pub async fn start() -> Result<(), DataError> {
  let app = app_builder::<config::Config>(None).run().await?;

  // 注册 Jieyuan 客户端
  let jieyuan_client = jieyuan_core::web::client::JieyuanClient::new()?;
  app.add_component(jieyuan_client);

  // 启动用户同步服务
  let user_sync_svc = UserSyncSvc::new(app.component())?;
  let shutdown_rx = app.shutdown_recv().await;
  tokio::spawn(async move {
    user_sync_svc.start_periodic_sync(shutdown_rx).await?;
    Ok::<(), DataError>(())
  });

  // 初始化 web
  init_web(app).await
}
