use std::collections::HashMap;
use std::sync::Arc;

use log::{error, info, warn};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use hetuflow_core::protocol::{GatewayCommand, SchedulerEvent, TaskInstanceUpdated};
use hetuflow_core::types::{ResourceLimits, TaskInstanceStatus, TaskStatus};

use crate::service::{
  ConnectionManager, ConnectionManagerConfig, ProcessManager, ProcessManagerConfig, RetryConfig, TaskExecutor,
  TaskExecutorConfig, TaskScheduler, TaskSchedulerConfig,
};
use crate::setting::{AgentConfig, AgentSetting, BackoffStrategy};

/// Agent 应用程序配置
#[derive(Debug, Clone)]
pub struct AgentApplicationConfig {
  pub agent: AgentConfig,
  pub connection: ConnectionManagerConfig,
  pub task_executor: TaskExecutorConfig,
  pub task_scheduler: TaskSchedulerConfig,
  pub process_manager: ProcessManagerConfig,
}

impl Default for AgentApplicationConfig {
  fn default() -> Self {
    Self {
      agent: AgentConfig {
        name: "hetuflow-agent".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["shell".to_string(), "python".to_string()],
        tags: HashMap::default(),
        work_dir: "/tmp/hetumind-agent/runs".to_string(),
        env_vars: Default::default(),
      },
      connection: ConnectionManagerConfig::default(),
      task_executor: TaskExecutorConfig::default(),
      task_scheduler: TaskSchedulerConfig::default(),
      process_manager: ProcessManagerConfig::default(),
    }
  }
}

/// Agent 应用程序
#[derive(Clone)]
pub struct AgentApplication {
  config: AgentApplicationConfig,
  connection_manager: Arc<ConnectionManager>,
  task_scheduler: Arc<TaskScheduler>,
  task_executor: Arc<TaskExecutor>,
  process_manager: Arc<ProcessManager>,
  shutdown_sender: Arc<tokio::sync::RwLock<Option<oneshot::Sender<()>>>>,
}

impl AgentApplication {
  /// 创建新的 Agent 应用程序
  pub fn new(config: AgentApplicationConfig) -> Self {
    info!("Creating AgentApplication with agent_id: {}", config.agent.name);

    // TODO:
    let agent_id = Uuid::now_v7();

    // 创建组件
    let connection_manager = Arc::new(ConnectionManager::new());
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let task_scheduler = Arc::new(TaskScheduler::new(config.task_scheduler.clone()));
    let process_manager = Arc::new(ProcessManager::new(config.process_manager.clone()));
    let task_executor = Arc::new(TaskExecutor::new(agent_id, process_manager.clone(), event_tx, None));

    let (shutdown_sender, _) = oneshot::channel();

    Self {
      config,
      connection_manager,
      task_scheduler,
      task_executor,
      process_manager,
      shutdown_sender: Arc::new(tokio::sync::RwLock::new(Some(shutdown_sender))),
    }
  }

  /// 从 AgentSetting 创建应用程序
  pub fn from_setting(setting: AgentSetting) -> Self {
    let mut config = AgentApplicationConfig::default();

    // 从 setting 更新配置
    config.connection.heartbeat_interval_seconds = setting.heartbeat_interval as u64;
    config.task_executor.default_task_timeout_seconds = setting.task_timeout as u64;

    // 从 settings 中提取其他配置
    if let Some(gateway_url) = setting.settings.get("gateway_url") {
      config.connection.gateway_url = gateway_url.clone();
    }

    if let Some(agent_name) = setting.settings.get("agent_name") {
      config.agent.name = agent_name.clone();
    }

    if let Some(max_concurrent_tasks) = setting.settings.get("max_concurrent_tasks") {
      if let Ok(value) = max_concurrent_tasks.parse::<usize>() {
        config.task_executor.max_concurrent_tasks = value;
        config.task_scheduler.max_concurrent_tasks = value;
      }
    }

    Self::new(config)
  }

  /// 获取 Agent ID
  pub fn get_agent_id(&self) -> Uuid {
    self.config.agent.agent_id
  }

  /// 获取 Agent 名称
  pub fn get_agent_name(&self) -> &str {
    &self.config.agent.name
  }

  /// 启动应用程序
  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting AgentApplication: {}", self.config.agent.name);

    // 启动 ProcessManager
    info!("Starting ProcessManager");
    self.process_manager.start().await?;

    // 启动 TaskExecutor
    info!("Starting TaskExecutor");
    let executor_command_receiver = self.task_executor.take_command_receiver().await;
    self.task_executor.start().await?;

    // 启动 TaskScheduler
    info!("Starting TaskScheduler");
    let scheduler_command_receiver = self.task_scheduler.take_command_receiver().await;
    self.task_scheduler.start().await?;

    // 设置组件间的通信
    self.setup_component_communication().await?;

    // 启动 ConnectionManager
    info!("Starting ConnectionManager");
    let connection_command_receiver = self.connection_manager.take_command_receiver().await;
    self.connection_manager.start().await?;

    // 启动命令处理循环
    self
      .start_command_processing_loops(
        connection_command_receiver,
        scheduler_command_receiver,
        executor_command_receiver,
      )
      .await;

    info!("AgentApplication started successfully");
    Ok(())
  }

  /// 设置组件间的通信
  async fn setup_component_communication(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TaskExecutor -> ConnectionManager (任务状态更新)
    let task_update_sender = self.connection_manager.get_command_sender().await;
    // TODO: 实现 TaskExecutor 的状态更新发送

    // TaskScheduler -> ConnectionManager (调度事件)
    let scheduler_event_sender = self.connection_manager.get_command_sender().await;
    // TODO: 实现 TaskScheduler 的事件发送

    // TaskScheduler -> TaskExecutor (任务执行命令)
    let executor_command_sender = self.task_executor.get_command_sender().await;
    // TODO: 实现 TaskScheduler 到 TaskExecutor 的命令发送

    Ok(())
  }

  /// 启动命令处理循环
  async fn start_command_processing_loops(
    &self,
    mut connection_command_receiver: mpsc::UnboundedReceiver<GatewayCommand>,
    mut scheduler_command_receiver: mpsc::UnboundedReceiver<SchedulerEvent>,
    mut executor_command_receiver: mpsc::UnboundedReceiver<TaskInstanceUpdated>,
  ) {
    let task_scheduler = Arc::clone(&self.task_scheduler);
    let task_executor = Arc::clone(&self.task_executor);
    let connection_manager = Arc::clone(&self.connection_manager);

    // 处理来自 Gateway 的命令
    tokio::spawn(async move {
      info!("Starting gateway command processing loop");

      while let Some(command) = connection_command_receiver.recv().await {
        match command {
          GatewayCommand::ExecuteTask { task_instance } => {
            info!("Received execute task command: {}", task_instance.task_instance_id);

            // 转发给 TaskScheduler
            if let Err(e) = task_scheduler.add_pending_task(task_instance).await {
              error!("Failed to add task to scheduler: {}", e);
            }
          }
          GatewayCommand::KillTask { task_instance_id } => {
            info!("Received kill task command: {}", task_instance_id);

            // 转发给 TaskExecutor
            if let Err(e) = task_executor
              .get_command_sender()
              .await
              .send(hetuflow_core::types::ExecutorCommand::KillTask { task_instance_id })
            {
              error!("Failed to send kill command to executor: {}", e);
            }
          }
          GatewayCommand::GetAgentStatus => {
            info!("Received get agent status command");
            // TODO: 实现状态收集和响应
          }
          GatewayCommand::UpdateConfiguration { config } => {
            info!("Received update configuration command");
            // TODO: 实现配置更新
            warn!("Configuration update not implemented: {:?}", config);
          }
        }
      }

      info!("Gateway command processing loop ended");
    });

    // 处理调度器事件
    tokio::spawn(async move {
      info!("Starting scheduler event processing loop");

      while let Some(event) = scheduler_command_receiver.recv().await {
        match event {
          SchedulerEvent::TaskScheduled { task_instance } => {
            info!("Task scheduled: {}", task_instance.task_instance_id);

            // 转发给 TaskExecutor
            if let Err(e) = task_executor
              .get_command_sender()
              .await
              .send(hetuflow_core::types::ExecutorCommand::ExecuteTask { task_instance })
            {
              error!("Failed to send execute command to executor: {}", e);
            }
          }
          SchedulerEvent::TaskRejected { task_instance_id, reason } => {
            warn!("Task rejected: {} - {}", task_instance_id, reason);

            // 通知 Gateway
            // TODO: 实现任务拒绝通知
          }
          SchedulerEvent::CapacityUpdated { available_capacity } => {
            info!("Capacity updated: {}", available_capacity);

            // 通知 Gateway
            // TODO: 实现容量更新通知
          }
        }
      }

      info!("Scheduler event processing loop ended");
    });

    // 处理执行器状态更新
    tokio::spawn(async move {
      info!("Starting executor status processing loop");

      while let Some(update) = executor_command_receiver.recv().await {
        info!("Task status updated: {} -> {:?}", update.task_instance_id, update.status);

        // 转发给 ConnectionManager
        // TODO: 实现状态更新转发
      }

      info!("Executor status processing loop ended");
    });
  }

  /// 等待关闭信号
  pub async fn wait_for_shutdown(&self) {
    let mut shutdown_receiver = {
      let mut sender_guard = self.shutdown_sender.write().await;
      if let Some(sender) = sender_guard.take() {
        let (new_sender, receiver) = oneshot::channel();
        *sender_guard = Some(new_sender);
        receiver
      } else {
        let (sender, receiver) = oneshot::channel();
        *sender_guard = Some(sender);
        receiver
      }
    };

    let _ = shutdown_receiver.await;
  }

  /// 停止应用程序
  pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Stopping AgentApplication: {}", self.config.agent.name);

    // 发送关闭信号
    if let Some(sender) = {
      let mut sender_guard = self.shutdown_sender.write().await;
      sender_guard.take()
    } {
      let _ = sender.send(());
    }

    // 停止各个组件
    info!("Stopping ConnectionManager");
    if let Err(e) = self.connection_manager.stop().await {
      error!("Failed to stop ConnectionManager: {}", e);
    }

    info!("Stopping TaskScheduler");
    // TODO: 实现 TaskScheduler 的停止方法

    info!("Stopping TaskExecutor");
    // TODO: 实现 TaskExecutor 的停止方法

    info!("Stopping ProcessManager");
    // TODO: 实现 ProcessManager 的停止方法

    info!("AgentApplication stopped");
    Ok(())
  }

  /// 获取应用程序状态
  pub async fn get_status(&self) -> AgentApplicationStatus {
    let connection_state = self.connection_manager.get_connection_state().await;
    let connection_stats = self.connection_manager.get_connection_stats().await;
    let scheduler_state = self.task_scheduler.get_state().await;
    let executor_task_count = self.task_executor.get_executing_task_count().await;

    AgentApplicationStatus {
      agent_id: self.config.agent.agent_id,
      agent_name: self.config.agent.name.clone(),
      connection_state,
      connection_stats,
      scheduler_state,
      executing_task_count: executor_task_count,
    }
  }
}

/// Agent 应用程序状态
#[derive(Debug, Clone)]
pub struct AgentApplicationStatus {
  pub agent_id: Uuid,
  pub agent_name: String,
  pub connection_state: hetuflow_core::types::ConnectionState,
  pub connection_stats: hetuflow_core::types::ConnectionStats,
  pub scheduler_state: hetuflow_core::types::TaskSchedulingState,
  pub executing_task_count: usize,
}
