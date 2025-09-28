use std::sync::Arc;

use fusion_common::{
  ahash::{HashMap, HashSet},
  time::datetime_from_millis,
};
use fusion_core::{
  DataError,
  concurrent::{ServiceHandle, ServiceTask, TaskResult},
};
use log::{error, info, warn};
use mea::{mpsc, shutdown::ShutdownRecv};
use modelsql::{ModelManager, filter::OpValsUuid};

use hetuflow_core::{
  models::*,
  protocol::{AcquireTaskRequest, AcquireTaskResponse, CommandMessage, ScheduledTask, TaskInstanceChanged},
  types::{TaskInstanceStatus, TaskStatus},
};

use crate::{
  connection::ConnectionManager, infra::bmc::*, model::AgentEvent, service::AgentSvc, setting::HetuflowSetting,
};

/// Agent 管理器 - 负责调度策略、可靠性统计和任务分发
pub struct AgentManager {
  mm: ModelManager,
  connection_manager: Arc<ConnectionManager>,
  setting: Arc<HetuflowSetting>,
}

impl AgentManager {
  /// 创建新的 Agent 管理器
  pub fn new(mm: ModelManager, connection_manager: Arc<ConnectionManager>, setting: Arc<HetuflowSetting>) -> Self {
    Self { mm, connection_manager, setting }
  }

  /// 运行 Agent 管理器（订阅事件流）
  pub async fn start(
    &self,
    shutdown_rx: ShutdownRecv,
  ) -> Result<Vec<ServiceHandle<Result<TaskResult, DataError>>>, DataError> {
    info!("Starting AgentManager with event subscription");
    // 订阅 Agent 事件
    let (event_tx, event_rx) = mpsc::unbounded();
    self.connection_manager.subscribe_event(event_tx).await?;

    let mut handles = Vec::new();

    handles.push(self.run_agent_cleanup(shutdown_rx.clone()));

    let agent_event_runner = AgentEventRunner {
      mm: self.mm.clone(),
      connection_manager: self.connection_manager.clone(),
      setting: self.setting.clone(),
      event_rx,
      shutdown_rx,
    };
    handles.push(agent_event_runner.start());

    Ok(handles)
  }

  // Agent connection timeout cleanup
  fn run_agent_cleanup(&self, shutdown_rx: ShutdownRecv) -> ServiceHandle<Result<TaskResult, DataError>> {
    let connection_manager = self.connection_manager.clone();
    let agent_heartbeat_ttl = self.setting.server.agent_overdue_ttl;
    let handle = tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
      loop {
        tokio::select! {
          _ = interval.tick() => { /* do nothing */ }
          _ = shutdown_rx.is_shutdown() => {
            info!("Shutdown signal received, stopping agent heartbeat timeout check loop.");
            break;
          }
        }

        if let Err(e) = connection_manager.cleanup_stale_connections(agent_heartbeat_ttl).await {
          error!("Connection cleanup failed: {:?}", e);
        }
      }
      Ok(TaskResult::new((), 0))
    });

    ServiceHandle::new("AgentCleanRunner", handle)
  }
}

struct AgentEventRunner {
  mm: ModelManager,
  connection_manager: Arc<ConnectionManager>,
  setting: Arc<HetuflowSetting>,
  event_rx: mpsc::UnboundedReceiver<AgentEvent>,
  shutdown_rx: ShutdownRecv,
}

impl ServiceTask<()> for AgentEventRunner {
  /// 处理 Agent 事件
  async fn run_loop(&mut self) -> Result<(), DataError> {
    let agent_svc = match AgentSvc::new_with_setting(self.mm.clone(), &self.setting) {
      Ok(svc) => svc,
      Err(e) => {
        error!("Failed to create AgentSvc with JWE config: {:?}", e);
        return Err(e);
      }
    };
    loop {
      tokio::select! {
        _ = self.shutdown_rx.is_shutdown() => {
          info!("Shutdown signal received, stopping agent event runner loop.");
          break;
        }
        event = self.event_rx.recv() => {
          if let Some(event) = event {
            self.process_event(&agent_svc, event).await;
          } else {
            info!("AgentEventRunner event channel closed, stopping loop.");
            break;
          }
        }
      }
    }

    Ok(())
  }
}

impl AgentEventRunner {
  /// Agent poll task 时不对 Server 绑定的 Namespace 进行过滤，直接拉取符合要求的最紧急的 SchedTaskInstance。按 request 条件进行过滤
  async fn process_task_poll(&self, agent_id: &str, request: Arc<AcquireTaskRequest>) -> Result<(), DataError> {
    info!("Agent {} task poll request: {:?}", agent_id, request);
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    let task_instances = TaskInstanceBmc::find_many_by_poll(&mm, &request).await?;
    let task_map = TaskBmc::find_many(
      &mm,
      vec![TaskFilter {
        id: Some(OpValsUuid::in_(task_instances.iter().map(|ti| ti.task_id).collect::<HashSet<_>>())),
        ..Default::default()
      }],
      None,
    )
    .await?
    .into_iter()
    .map(|t| (t.id, t))
    .collect::<HashMap<_, _>>();

    let tasks = task_instances
      .into_iter()
      .filter_map(|task_instance| {
        let task_id = task_instance.task_id;
        match task_map.get(&task_id) {
          Some(task) => Some(ScheduledTask { task_instance, task: task.clone() }),
          None => {
            warn!("SchedTask({}) of SchedTaskInstance({}) not exists", task_id, task_instance.id);
            None
          }
        }
      })
      .collect::<Vec<_>>();

    // 向 Agent 发送 TaskPollResponse
    let payload = AcquireTaskResponse { tasks, has_more: false, next_poll_interval: 0 };
    let command = CommandMessage::new_acquire_task(payload);
    self.connection_manager.send_to_agent(agent_id, command).await?;

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  async fn process_task_instance_changed(
    &self,
    agent_id: &str,
    payload: Arc<TaskInstanceChanged>,
  ) -> Result<(), DataError> {
    info!("Processing task instance changed for agent {}, instance {}", agent_id, payload.instance_id);

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    let task_instance = TaskInstanceBmc::find_by_id(&mm, &payload.instance_id).await?;

    // 1. 更新任务实例状态
    let instance_update = TaskInstanceForUpdate {
      status: Some(payload.status),
      started_at: if payload.status == TaskInstanceStatus::Running && task_instance.started_at.is_none() {
        Some(datetime_from_millis(payload.epoch_millis))
      } else {
        None
      },
      completed_at: if matches!(
        payload.status,
        TaskInstanceStatus::Succeeded | TaskInstanceStatus::Failed | TaskInstanceStatus::Cancelled
      ) {
        Some(datetime_from_millis(payload.epoch_millis))
      } else {
        None
      },
      error_message: payload.error_message.clone(),
      output: payload.data.clone(),
      ..Default::default()
    };

    info!("Instance updated is {:?}", instance_update);
    TaskInstanceBmc::update_by_id(&mm, payload.instance_id, instance_update).await?;

    // 2. 获取更新后的任务实例信息以获取关联的任务ID
    let task_instance = TaskInstanceBmc::find_by_id(&mm, &payload.instance_id).await?;

    // 3. 如果任务失败，需要更新任务的重试计数和状态
    if payload.status == TaskInstanceStatus::Failed {
      // 获取当前任务信息
      let current_task = TaskBmc::find_by_id(&mm, &task_instance.task_id).await?;

      let new_retry_count = current_task.retry_count + 1;
      let max_retries = current_task.config.max_retries as i32;
      let new_status = if new_retry_count >= max_retries {
        TaskStatus::Failed // 达到最大重试次数，标记为最终失败
      } else {
        TaskStatus::Pending // 还可以重试，重置为待处理状态
      };

      // 更新任务状态和重试计数
      let task_update =
        TaskForUpdate { status: Some(new_status), retry_count: Some(new_retry_count), ..Default::default() };

      TaskBmc::update_by_id(&mm, task_instance.task_id, task_update).await?;

      info!("Updated task {} retry count to {}, status: {:?}", task_instance.task_id, new_retry_count, new_status);
    } else if payload.status == TaskInstanceStatus::Succeeded {
      // 任务成功完成，更新任务状态
      let task_update = TaskForUpdate { status: Some(TaskStatus::Succeeded), ..Default::default() };

      TaskBmc::update_by_id(&mm, task_instance.task_id, task_update).await?;
      info!("Task {} completed successfully", task_instance.task_id);
    }

    // 4. 更新 Agent 统计信息
    let success = payload.status == TaskInstanceStatus::Succeeded;

    let response_time_ms = if let Some(completed_at) = task_instance.completed_at
      && let Some(started_at) = task_instance.started_at
    {
      (completed_at - started_at).num_milliseconds()
    } else {
      0
    };

    if let Some(agent) = self.connection_manager.get_agent(agent_id).await? {
      agent.update_stats(success, response_time_ms).await;
    }

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  async fn process_event(&mut self, agent_svc: &AgentSvc, event: AgentEvent) {
    match event {
      AgentEvent::Heartbeat { agent_id, .. } => {
        if let Err(e) = agent_svc.update_agent_heartbeat(&agent_id).await {
          error!("Failed to update agent heartbeat agent {}: {:?}", agent_id, e);
        }
      }
      AgentEvent::TaskInstanceChanged { agent_id, payload } => {
        if let Err(e) = self.process_task_instance_changed(&agent_id, payload).await {
          error!("Failed to process task instance changed agent {}: {:?}", agent_id, e);
        }
      }
      AgentEvent::TaskPollRequest { agent_id, request } => {
        if let Err(e) = self.process_task_poll(&agent_id, request).await {
          error!("Failed to process task poll request agent {}: {:?}", agent_id, e);
        }
      }
      AgentEvent::Registered { agent_id, payload } => match agent_svc.handle_register(&agent_id, &payload).await {
        Ok(response) => {
          let message = CommandMessage::new_agent_registered(response);
          if let Err(e) = self.connection_manager.send_to_agent(&agent_id, message).await {
            error!("Failed to send registered message to agent {}: {:?}", agent_id, e);
          }
        }
        Err(e) => {
          error!("Failed to handle register agent {}: {:?}", agent_id, e);
        }
      },
      AgentEvent::Connected { .. } => {
        // do nothing
      }
      AgentEvent::Unconnected { agent_id, reason } => {
        warn!("Agent {} disconnected: {}", agent_id, reason);
        // 处理 Agent 离线导致的任务失败
        if let Err(e) = agent_svc.handle_agent_offline(&agent_id).await {
          error!("Failed to handle offline agent {}: {:?}", agent_id, e);
        }
      }
      AgentEvent::TaskLog { .. } => { /* do nothing */ }
    }
  }
}
