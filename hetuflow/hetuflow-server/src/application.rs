use std::{sync::Arc, time::Duration};

use fusion_core::{
  DataError,
  application::Application,
  concurrent::{ServiceHandle, ServiceTask, TaskResult, TaskServiceHandle},
};
use fusion_db::DbPlugin;
use fusion_web::server::WebServerBuilder;
use fusionsql::ModelManager;
use log::{error, info};
use mea::{mutex::Mutex, shutdown::ShutdownRecv};

use crate::infra::bmc::AgentBmc;
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
  pub log_svc: LogSvc,
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

    let connection_manager = Arc::new(ConnectionManager::new());

    let message_handler = Arc::new(MessageHandler::new(connection_manager.clone()));

    let agent_manager =
      Arc::new(AgentManager::new(application.component(), connection_manager.clone(), setting.clone()));

    let log_svc =
      LogSvc::new(Arc::new(setting.task_log.clone()), application.shutdown_recv().await, connection_manager.clone())
        .await?;

    let broker = Broker::new(setting.clone(), application.component());

    Ok(Self {
      setting,
      broker,
      connection_manager,
      message_handler,
      agent_manager,
      log_svc,
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
    Application::global().shutdown_recv().await
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
      match WebServerBuilder::new(router).with_shutdown(shutdown_rx).build().await {
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
    // 发送关闭信号
    Application::shutdown().await;

    drop(self.connection_manager);
    drop(self.agent_manager);
    drop(self.log_svc);

    Application::await_shutdown().await;

    info!("Waiting for all service tasks to complete");
    let mut handles_guard = self.handles.lock().await;
    let handles = std::mem::take(&mut *handles_guard);
    for handle in handles {
      let name = handle.name().to_string();
      let ret = tokio::time::timeout(Duration::from_secs(10), handle.complete()).await;
      match ret {
        Ok(Ok((name, _))) => {
          info!("Task name: {} completed successfully", name);
        }
        Ok(Err((name, e))) => {
          error!("Failed to join task name: {}, error: {}", name, e);
        }
        Err(e) => {
          error!("Waiting for join ServiceHandle name: {} timeout, error: {}", name, e);
        }
      }
    }

    info!("Hetuflow Application shutdown complete");
    Ok(())
  }

  pub async fn health_status(&self) -> Result<SystemStatus, DataError> {
    let mm = self.mm();
    let db = mm.dbx().db_postgres()?;
    let db_size = db.db().size();

    // 从数据库获取在线 Agent 数量，而不是从内存连接管理器
    let online_agents = AgentBmc::find_online_agents(&mm)
      .await
      .map_err(|e| DataError::internal(500, "Failed to get online agents", Some(Box::new(e))))?;
    let agent_size = online_agents.len() as u32;

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
