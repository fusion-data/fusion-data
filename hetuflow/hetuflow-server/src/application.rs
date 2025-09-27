use std::sync::Arc;

use fusion_core::{
  DataError,
  application::Application,
  concurrent::{ServiceHandle, ServiceTask, TaskResult, TaskServiceHandle},
  configuration::ConfigRegistry,
};
use fusion_db::DbPlugin;
use log::{error, info};
use mea::{
  mutex::Mutex,
  shutdown::{ShutdownRecv, ShutdownSend},
};
use modelsql::ModelManager;

use crate::{
  broker::Broker,
  connection::{ConnectionManager, MessageHandler},
  scheduler::TaskGenerationRunner,
  service::{AgentManager, LogSvc},
};
use crate::{model::SystemStatus, setting::HetuflowSetting};

#[derive(Clone)]
pub struct ServerApplication {
  pub setting: Arc<HetuflowSetting>,
  pub broker: Broker,
  pub connection_manager: Arc<ConnectionManager>,
  pub message_handler: Arc<MessageHandler>,
  agent_manager: Arc<AgentManager>,
  log_svc: Arc<LogSvc>,
  shutdown: Arc<Mutex<Option<(ShutdownSend, ShutdownRecv)>>>,
  handles: Arc<Mutex<Vec<TaskServiceHandle>>>,
}

impl ServerApplication {
  pub async fn new() -> Result<Self, DataError> {
    Self::new_with_source::<config::Environment>(None).await
  }

  pub async fn new_with_source<S>(config_source: Option<S>) -> Result<Self, DataError>
  where
    S: config::Source + Send + Sync + 'static,
  {
    let application = Application::builder().add_plugin(DbPlugin).run().await?;

    if let Some(config_source) = config_source {
      let config_registry = application.config_registry();
      config_registry.add_config_source(config_source)?;
      config_registry.reload()?;
    }

    let setting = Arc::new(HetuflowSetting::load(application.config_registry())?);

    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();

    let connection_manager = Arc::new(ConnectionManager::new());

    let message_handler = Arc::new(MessageHandler::new(connection_manager.clone()));

    let agent_manager =
      Arc::new(AgentManager::new(application.component(), connection_manager.clone(), setting.clone()));

    let log_receiver =
      Arc::new(LogSvc::new(Arc::new(setting.task_log.clone()), shutdown_rx.clone(), connection_manager.clone()).await?);

    let broker = Broker::new(setting.clone(), application.component());

    Ok(Self {
      setting,
      broker,
      connection_manager,
      message_handler,
      agent_manager,
      log_svc: log_receiver,
      shutdown: Arc::new(Mutex::new(Some((shutdown_tx, shutdown_rx)))),
      handles: Arc::new(Mutex::new(Vec::new())),
    })
  }

  /// 启动调度器应用
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting Hetuflow Application");

    // 1. 启动 Broker 服务
    self.broker.start(self.get_shutdown_recv().await).await?;

    // 2. 启动通用服务 (所有实例都启动)
    self.start_common_services().await?;

    info!("Hetuflow Application started successfully");
    Ok(())
  }

  pub async fn get_shutdown_recv(&self) -> ShutdownRecv {
    let maybe = self.shutdown.lock().await;
    let tuple = maybe.as_ref().unwrap();
    tuple.1.clone()
  }

  /// 启动通用服务
  async fn start_common_services(&self) -> Result<(), DataError> {
    info!("Starting common services");
    let mut handles = self.handles.lock().await;

    // 启用 Scheduler 服务，根据 Time/Cron 类型的调度生成 SchedTask/SchedTaskInstance
    let task_generation_runner =
      TaskGenerationRunner::new(self.setting.clone(), self.mm(), self.get_shutdown_recv().await);
    handles.push(task_generation_runner.start());

    // 启动 Agent 管理器（事件订阅）
    handles.extend(self.agent_manager.start(self.get_shutdown_recv().await).await?);

    // // 启动网关服务
    // self.gateway_svc.start().await?;

    // 处理从 Agent 上报的日志
    self.log_svc.start().await?;

    // 启动 HTTP API 服务 (/api/v1)
    let shutdown_rx = self.get_shutdown_recv().await;
    let router = crate::endpoint::routes().with_state(self.clone());
    let handle = tokio::spawn(async move {
      let conf = Application::global().get_config().expect("WebConfig not valid");
      match fusion_web::server::init_server_with_config(&conf, router, Some(shutdown_rx)).await {
        Ok(_) => {
          info!("HTTP server has been shutdown");
        }
        Err(e) => {
          error!("HTTP server init error: {:?}", e);
        }
      }
      Ok(TaskResult::empty())
    });
    handles.push(ServiceHandle::new("Web", handle));

    Ok(())
  }

  /// 优雅关闭
  pub async fn shutdown(self) -> Result<(), DataError> {
    info!("Shutting down Hetuflow Application");
    let shutdown_tx = match self.shutdown.lock().await.take() {
      Some((tx, _)) => tx, // discard ShutdownRecv
      None => return Err(DataError::server_error("AgentApplication is not running")),
    };

    // 发送关闭信号
    shutdown_tx.shutdown();

    drop(self.connection_manager);
    drop(self.agent_manager);
    drop(self.log_svc);

    shutdown_tx.await_shutdown().await;

    info!("Waiting for all service tasks to complete");
    let mut handles_guard = self.handles.lock().await;
    let handles = std::mem::take(&mut *handles_guard);
    for handle in handles {
      if let Err((name, e)) = handle.complete().await {
        error!("Failed to join task name: {}, error: {}", name, e);
      }
    }

    info!("Hetuflow Application shutdown complete");
    Ok(())
  }

  pub async fn health_status(&self) -> Result<SystemStatus, DataError> {
    let mm = self.mm();
    let db = mm.dbx().db_postgres()?;
    let db_size = db.db().size();

    let agent_size = self.connection_manager.get_online_count().await?;

    let body = SystemStatus::new(db_size, agent_size);
    Ok(body)
  }

  /// 获取应用配置
  pub fn setting(&self) -> &HetuflowSetting {
    &self.setting
  }

  pub fn mm(&self) -> ModelManager {
    Application::global().component()
  }
}
