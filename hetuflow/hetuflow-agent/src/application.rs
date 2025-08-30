use std::sync::Arc;

use log::{error, info};
use tokio::sync::{broadcast, mpsc};
use ultimate_core::DataError;
use ultimate_core::application::Application;
use ultimate_core::timer::{Timer, TimerPlugin};
use uuid::Uuid;

use crate::service::{ConnectionManager, ProcessManager, TaskExecutor, TaskScheduler};
use crate::setting::HetuflowAgentSetting;

/// Agent 应用程序
#[derive(Clone)]
pub struct AgentApplication {
  setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  task_scheduler: Arc<TaskScheduler>,
  task_executor: Arc<TaskExecutor>,
  process_manager: Arc<ProcessManager>,
  shutdown_tx: broadcast::Sender<()>,
}

impl AgentApplication {
  pub async fn new() -> Result<Self, DataError> {
    let application = Application::builder().add_plugin(TimerPlugin).run().await?;
    let setting: Arc<HetuflowAgentSetting> = Arc::new(HetuflowAgentSetting::load(application.config_registry())?);
    info!("Creating AgentApplication with agent_id: {}", setting.agent_id);

    let agent_id = setting.agent_id;

    let (poll_task_resp_tx, task_poll_resp_rx) = mpsc::unbounded_channel();
    let (shutdown_tx, _) = broadcast::channel(1);

    // 创建组件
    let connection_manager = Arc::new(ConnectionManager::new(setting.clone(), shutdown_tx.clone(), poll_task_resp_tx));
    let task_scheduler = Arc::new(TaskScheduler::new(
      setting.clone(),
      shutdown_tx.clone(),
      connection_manager.clone(),
      task_poll_resp_rx,
      application.component::<Timer>().timer_ref(),
    ));
    let process_manager = Arc::new(ProcessManager::new(setting.process.clone(), Arc::new(shutdown_tx.subscribe())));
    let task_executor = Arc::new(TaskExecutor::new(
      setting.clone(),
      shutdown_tx.clone(),
      process_manager.clone(),
      connection_manager.clone(),
      None,
    ));

    Ok(Self { setting, connection_manager, task_scheduler, task_executor, process_manager, shutdown_tx })
  }

  /// 获取 Agent ID
  pub fn get_agent_id(&self) -> Uuid {
    self.setting.agent_id
  }

  /// 启动应用程序
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting AgentApplication: {}", self.setting.agent_id);

    // 启动 ConnectionManager
    info!("Starting ConnectionManager");
    self.connection_manager.start().await?;

    // 启动 ProcessManager
    info!("Starting ProcessManager");
    self.process_manager.start().await?;

    // 启动 TaskExecutor
    info!("Starting TaskExecutor");
    self.task_executor.start().await?;

    // 启动 TaskScheduler
    info!("Starting TaskScheduler");
    self.task_scheduler.start().await?;

    info!("AgentApplication started successfully");
    Ok(())
  }

  /// 停止应用程序
  pub async fn shutdown(&self) -> Result<(), DataError> {
    info!("AgentApplication shutdown begging: {}", self.setting.agent_id);

    // 发送关闭信号
    if let Err(e) = self.shutdown_tx.send(()) {
      error!("Shutdown error: {}", e);
    }

    info!("Stopping TaskScheduler");
    // TODO: 实现 TaskScheduler 的停止方法

    info!("Stopping TaskExecutor");
    // TODO: 实现 TaskExecutor 的停止方法

    info!("Stopping ProcessManager");
    // TODO: 实现 ProcessManager 的停止方法

    // 停止各个组件
    if let Err(e) = self.connection_manager.wait_closed().await {
      error!("Wait closed ConnectionManager error: {}", e);
    } else {
      info!("Wait closed ConnectionManager successfully");
    }

    info!("AgentApplication shutdown successfully");
    Ok(())
  }
}
