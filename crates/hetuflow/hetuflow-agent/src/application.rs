use std::sync::Arc;

use fusion_core::DataError;
use fusion_core::application::Application;
use fusion_core::concurrent::handle::ServiceHandle;
use fusion_core::timer::{Timer, TimerPlugin};
use log::info;
use mea::mutex::Mutex;
use mea::shutdown::{ShutdownRecv, ShutdownSend};

use crate::connection::{CommandRunner, ConnectionManager, WsRunner};
use crate::process::{ProcessCleanupRunner, ProcessManager};
use crate::service::{TaskExecutor, TaskScheduler};
use crate::setting::HetuflowAgentSetting;

/// Agent 应用程序
#[derive(Clone)]
pub struct AgentApplication {
  pub setting: Arc<HetuflowAgentSetting>,
  connection_manager: Arc<ConnectionManager>,
  task_scheduler: Arc<TaskScheduler>,
  process_manager: Arc<ProcessManager>,
  task_executor: Arc<TaskExecutor>,
  shutdown: Arc<Mutex<Option<(ShutdownSend, ShutdownRecv)>>>,
  handles: Arc<Mutex<Vec<ServiceHandle<()>>>>,
}

impl AgentApplication {
  pub async fn new() -> Result<Self, DataError> {
    Self::new_with_source::<config::Environment>(None).await
  }

  pub async fn new_with_source<S>(config_source: Option<S>) -> Result<Self, DataError>
  where
    S: config::Source + Send + Sync + 'static,
  {
    let application = Application::builder().add_plugin(TimerPlugin).run().await?;
    if let Some(config_source) = config_source {
      let config_registry = application.config_registry();
      config_registry.add_config_source(config_source)?;
      config_registry.reload()?;
    }

    let setting: Arc<HetuflowAgentSetting> = Arc::new(HetuflowAgentSetting::load(application.config_registry())?);
    info!("Creating AgentApplication with agent_id: {}", setting.agent_id);

    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();

    let connection_manager = Arc::new(ConnectionManager::new());
    let process_manager = Arc::new(ProcessManager::new(setting.process.clone(), connection_manager.clone()));
    let task_scheduler = Arc::new(TaskScheduler::new(
      setting.clone(),
      process_manager.clone(),
      shutdown_rx.clone(),
      connection_manager.clone(),
      application.component::<Timer>().timer_ref(),
    ));
    let task_executor = Arc::new(TaskExecutor::new(
      setting.clone(),
      process_manager.clone(),
      connection_manager.clone(),
      task_scheduler.scheduled_task_rx(),
      shutdown_rx.clone(),
    ));
    Ok(Self {
      setting,
      connection_manager,
      task_scheduler,
      process_manager,
      task_executor,
      shutdown: Arc::new(Mutex::new(Some((shutdown_tx, shutdown_rx)))),
      handles: Arc::new(Mutex::new(Vec::new())),
    })
  }

  /// 获取 Agent ID
  pub fn get_agent_id(&self) -> &str {
    &self.setting.agent_id
  }

  /// 启动应用程序
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting AgentApplication: {}", self.setting.agent_id);

    let mut handles = self.handles.lock().await;
    handles.extend(self.task_executor.start()?);
    handles.extend(self.task_scheduler.start()?);

    let shutdown_rx = self.shutdown_recv().await;

    let ws_runner = WsRunner::new(self.setting.clone(), self.connection_manager.clone(), shutdown_rx.clone());
    handles.push(ServiceHandle::new("WsRunner", tokio::spawn(async move { ws_runner.run_loop().await })));

    let command_runner = CommandRunner::new(self.connection_manager.clone(), shutdown_rx.clone());
    handles.push(ServiceHandle::new("CommandRunner", tokio::spawn(async move { command_runner.run_loop().await })));

    let process_cleanup_runner = ProcessCleanupRunner::new(self.process_manager.clone(), shutdown_rx.clone());
    handles.push(ServiceHandle::new(
      "ProcessCleanupRunner",
      tokio::spawn(async move { process_cleanup_runner.run_loop().await }),
    ));

    info!("AgentApplication started successfully");
    Ok(())
  }

  pub async fn shutdown_recv(&self) -> ShutdownRecv {
    let guard = self.shutdown.lock().await;
    guard.as_ref().unwrap().1.clone()
  }

  /// 停止应用程序
  pub async fn shutdown(self) -> Result<(), DataError> {
    info!("AgentApplication shutdown begging, agent_id: {}", self.setting.agent_id);

    // 取出 ShutdownSend
    let shutdown_tx = match self.shutdown.lock().await.take() {
      Some((tx, _)) => tx, // discard ShutdownRecv
      None => return Err(DataError::server_error("AgentApplication is not running")),
    };

    // 发送关闭信号
    shutdown_tx.shutdown();

    drop(self.connection_manager);
    drop(self.task_scheduler);
    drop(self.process_manager);
    self.task_executor.stop().await?;
    drop(self.task_executor);

    // 等待各组件停止完成
    shutdown_tx.await_shutdown().await;

    // 等待所有任务完成
    let mut handles_guard = self.handles.lock().await;
    let handles = std::mem::take(&mut *handles_guard);
    for handle in handles {
      if let Err((svc_name, e)) = handle.await_complete().await {
        log::error!("Failed to join service name: {:?}, error: {:?}", svc_name, e);
      }
    }

    info!("AgentApplication shutdown successfully");
    Ok(())
  }
}
