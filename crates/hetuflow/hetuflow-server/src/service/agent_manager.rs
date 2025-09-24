use std::sync::Arc;

use fusion_common::{
  ahash::{HashMap, HashSet},
  time::{datetime_from_millis, now_offset},
};
use fusion_core::DataError;
use log::{debug, error, info, warn};
use mea::mpsc;
use modelsql::{ModelManager, filter::OpValsUuid};
use tokio::task::JoinHandle;

use hetuflow_core::{
  models::*,
  protocol::{AcquireTaskRequest, AcquireTaskResponse, ScheduledTask, TaskInstanceChanged, WebSocketCommand},
  types::{CommandKind, TaskInstanceStatus, TaskStatus},
};

use crate::{
  gateway::ConnectionManager,
  infra::bmc::*,
  model::{AgentEvent, AgentReliabilityStats},
  service::AgentSvc,
  setting::HetuflowSetting,
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
  pub async fn start(&self) -> Result<JoinHandle<()>, DataError> {
    info!("Starting AgentManager with event subscription");
    // 订阅 Agent 事件
    let (tx, event_receiver) = mpsc::unbounded();
    self.connection_manager.subscribe_event(tx).await?;

    let run_loop = AgentEventRunLoop {
      mm: self.mm.clone(),
      connection_manager: self.connection_manager.clone(),
      setting: self.setting.clone(),
      event_rx: event_receiver,
    };
    let join_handle = tokio::spawn(run_loop.run_loop());
    Ok(join_handle)
  }

  /// 检查 Agent 健康状态（现在主要关注任务清理）
  pub async fn check_agent_health(&self) -> Result<(), DataError> {
    debug!("Checking agent health and cleaning up zombie tasks");
    Ok(())
  }

  /// 获取在线 Agent 列表（通过 AgentRegistry）
  pub async fn get_agents(&self) -> Result<Vec<SchedAgent>, DataError> {
    let online_agents = self.connection_manager.get_online_agents().await?;
    if online_agents.is_empty() {
      return Ok(vec![]);
    }

    let agent_svc = AgentSvc::new(self.mm.clone());
    let agents = agent_svc.find_online_agents().await?;
    Ok(agents)
  }

  /// 获取 Agent 可靠性统计
  pub async fn get_agent_details(&self, agent_id: &str) -> Result<Option<AgentReliabilityStats>, DataError> {
    if let Some(agent) = self.connection_manager.get_agent(agent_id).await? {
      let stats = agent.stats().await;
      Ok(Some(stats))
    } else {
      Ok(None)
    }
  }

  /// 选择最佳 Agent（基于可靠性统计和负载均衡）
  pub async fn select_best_agent(
    &self,
    _task_requirements: Option<serde_json::Value>,
  ) -> Result<Option<String>, DataError> {
    let online_agents = self.connection_manager.get_online_agents().await?;

    if online_agents.is_empty() {
      return Ok(None);
    }

    let futures: Vec<JoinHandle<(String, AgentReliabilityStats)>> = online_agents
      .into_iter()
      .map(|agent| {
        let agent_id = agent.agent_id.clone();
        tokio::spawn(async move {
          let stats = agent.stats().await;
          (agent_id, stats)
        })
      })
      .collect();
    let agent_stats_vec = futures_util::future::join_all(futures).await;

    // 简单的负载均衡：选择连续失败次数最少、任务数最少的 Agent
    let best_agent = agent_stats_vec
      .iter()
      .filter_map(|result| result.as_ref().ok())
      .min_by_key(|(_, stats)| (stats.consecutive_failures, stats.total_tasks))
      .cloned();

    Ok(best_agent.map(|(agent_id, _)| agent_id))
  }

  /// 更新任务执行统计
  pub async fn update_task_stats(&self, agent_id: &str, success: bool, response_time_ms: f64) -> Result<(), DataError> {
    if let Some(agent) = self.connection_manager.get_agent(agent_id).await? {
      agent.update_stats(success, response_time_ms).await;
    }

    Ok(())
  }

  /// 刷新 Agent 状态（现在主要从 AgentRegistry 获取）
  pub async fn refresh_agent_status(&self) -> Result<(), DataError> {
    debug!("Refreshing agent status from AgentRegistry");

    // 获取当前在线 Agent 数量和列表，用于统计信息更新
    let online_count = self.connection_manager.get_online_count().await?;
    let online_agents = self.connection_manager.get_online_agents().await?;
    info!(
      "Current online agents: {}, agent list: {:?}",
      online_count,
      online_agents.iter().map(|agent| agent.agent_id.as_str()).collect::<Vec<_>>()
    );

    // TODO: 待实现

    Ok(())
  }

  /// 获取统计信息（包含可靠性数据）
  pub async fn get_stats(&self) -> Result<serde_json::Value, DataError> {
    let online_agents = self.connection_manager.get_online_agents().await?;
    let online_count = online_agents.len();

    let mut total_tasks: u64 = 0;
    let mut total_successes: u64 = 0;
    let mut agents_with_failures: u64 = 0;

    for agent in online_agents {
      let stats = agent.stats().await;
      total_tasks += stats.total_tasks;
      total_successes += stats.success_count;
      if stats.consecutive_failures > 0 {
        agents_with_failures += 1;
      }
    }

    let avg_success_rate = if total_tasks > 0 { total_successes as f64 / total_tasks as f64 } else { 0.0 };

    Ok(serde_json::json!({
      "online_agents": online_count,
      "total_tasks_processed": total_tasks,
      "overall_success_rate": avg_success_rate,
      "agents_with_failures": agents_with_failures
    }))
  }
}

struct AgentEventRunLoop {
  mm: ModelManager,
  connection_manager: Arc<ConnectionManager>,
  setting: Arc<HetuflowSetting>,
  event_rx: mpsc::UnboundedReceiver<AgentEvent>,
}

impl AgentEventRunLoop {
  /// 处理 Agent 事件
  async fn run_loop(mut self) {
    let agent_svc = match AgentSvc::new_with_setting(self.mm.clone(), &self.setting) {
      Ok(svc) => svc,
      Err(e) => {
        error!("Failed to create AgentSvc with JWE config: {:?}", e);
        return;
      }
    };
    while let Some(event) = self.event_rx.recv().await {
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
            let message = WebSocketCommand::new(CommandKind::AgentRegistered, response);
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
    info!("Find {} tasks for agent {}", tasks.len(), agent_id);
    let parameters = serde_json::to_value(AcquireTaskResponse { tasks, has_more: false, next_poll_interval: 0 })?;
    let command = WebSocketCommand::new(CommandKind::DispatchTask, parameters);
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

    // 1. 获取任务实例信息以获取关联的任务ID
    let task_instance = TaskInstanceBmc::find_by_id(&mm, &payload.instance_id).await?;

    // 2. 更新任务实例状态
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
    let response_time_ms = payload
      .metrics
      .as_ref()
      .and_then(|m| m.end_time.map(|end| (end - m.start_time) as f64))
      .unwrap_or(0.0);

    if let Some(agent) = self.connection_manager.get_agent(agent_id).await? {
      agent.update_stats(success, response_time_ms).await;
    }

    mm.dbx().commit_txn().await?;
    Ok(())
  }
}
