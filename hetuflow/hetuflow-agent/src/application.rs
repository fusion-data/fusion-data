use std::sync::{Arc, Mutex};

use fusion_core::DataError;
use fusion_core::application::Application;
use fusion_core::timer::{Timer, TimerPlugin};
use log::{error, info};
use mea::shutdown::{ShutdownRecv, ShutdownSend};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::service::{ConnectionManager, ProcessManager, TaskExecutor, TaskScheduler};
use crate::setting::HetuflowAgentSetting;

/// Agent 应用程序
#[derive(Clone)]
pub struct AgentApplication {
  setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  task_scheduler: Arc<TaskScheduler>,
  task_executors: Vec<Arc<TaskExecutor>>,
  process_manager: Arc<ProcessManager>,
  shutdown_tx: Arc<Mutex<Option<ShutdownSend>>>,
}

impl AgentApplication {
  pub async fn new() -> Result<Self, DataError> {
    let application = Application::builder().add_plugin(TimerPlugin).run().await?;
    let setting: Arc<HetuflowAgentSetting> = Arc::new(HetuflowAgentSetting::load(application.config_registry())?);
    info!("Creating AgentApplication with agent_id: {}", setting.agent_id);

    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();

    let (scheduled_task_tx, scheduled_task_rx) = kanal::unbounded_async();

    // 创建组件
    let connection_manager = Arc::new(ConnectionManager::new(setting.clone(), shutdown_rx.clone()));
    let task_scheduler = Arc::new(TaskScheduler::new(
      setting.clone(),
      shutdown_rx.clone(),
      connection_manager.clone(),
      application.component::<Timer>().timer_ref(),
      scheduled_task_tx,
    ));
    let process_manager = Arc::new(ProcessManager::new(setting.process.clone(), shutdown_rx.clone()));
    let task_executors = (0..setting.process.max_concurrent_processes)
      .map(|_i| {
        Arc::new(TaskExecutor::new(
          setting.clone(),
          process_manager.clone(),
          connection_manager.clone(),
          scheduled_task_rx.clone(),
        ))
      })
      .collect();

    Ok(Self {
      setting,
      connection_manager,
      task_scheduler,
      task_executors,
      process_manager,
      shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    })
  }

  /// 获取 Agent ID
  pub fn get_agent_id(&self) -> Uuid {
    self.setting.agent_id
  }

  /// 启动应用程序
  pub async fn start(&mut self) -> Result<(), DataError> {
    info!("Starting AgentApplication: {}", self.setting.agent_id);

    // 启动 ProcessManager
    info!("Starting ProcessManager");
    self.process_manager.start().await?;

    // 启动 TaskExecutor
    let task_executors = std::mem::take(&mut self.task_executors);
    for task_executor in task_executors {
      let _handle = tokio::spawn(async move { task_executor.run_loop().await });
    }

    // 启动 TaskScheduler
    info!("Starting TaskScheduler");
    self.task_scheduler.start().await?;

    // 启动 ConnectionManager
    info!("Starting ConnectionManager");
    self.connection_manager.start().await?;

    info!("AgentApplication started successfully");
    Ok(())
  }

  /// 停止应用程序
  pub async fn shutdown(&self) -> Result<(), DataError> {
    info!("AgentApplication shutdown begging: {}", self.setting.agent_id);
    let mut shutdown_tx = self.shutdown_tx.lock().unwrap();
    if shutdown_tx.is_none() {
      return Err(DataError::server_error("AgentApplication is not running"));
    }

    // 取出 ShutdownSend
    let shutdown_tx = shutdown_tx.take().unwrap();

    // 发送关闭信号
    shutdown_tx.shutdown();

    // 等待各组件停止完成
    shutdown_tx.await_shutdown().await;
    info!("AgentApplication shutdown successfully");
    Ok(())
  }
}
