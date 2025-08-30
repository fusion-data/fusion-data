use std::sync::Arc;

use hetuflow_core::{
  models::*,
  protocol::{
    AgentRegisterResponse, ScheduledTask, TaskInstanceUpdated, TaskPollRequest, TaskPollResponse, WebSocketCommand,
  },
  types::{AgentStatus, CommandKind, TaskInstanceStatus, TaskStatus},
};
use log::{debug, error, info, warn};
use modelsql::{
  ModelManager,
  filter::{OpValsInt32, OpValsUuid},
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use ultimate_common::{
  ahash::{HashMap, HashSet},
  time::{now_epoch_millis, now_offset},
};
use ultimate_core::DataError;
use uuid::Uuid;

use crate::{gateway::ConnectionManager, model::AgentEvent};
use crate::{infra::bmc::*, service::AgentSvc};

/// Agent 管理器 - 负责调度策略、可靠性统计和任务分发
pub struct AgentManager {
  mm: ModelManager,
  connection_manager: Arc<ConnectionManager>,
}

impl AgentManager {
  /// 创建新的 Agent 管理器
  pub fn new(mm: ModelManager, connection_manager: Arc<ConnectionManager>) -> Self {
    Self { mm, connection_manager }
  }

  /// 运行 Agent 管理器（订阅事件流）
  pub async fn start(&self) -> Result<JoinHandle<()>, DataError> {
    info!("Starting AgentManager with event subscription");
    // 订阅 Agent 事件
    let (tx, event_receiver) = mpsc::unbounded_channel();
    self.connection_manager.subscribe_event(tx)?;

    let run_loop = AgentEventRunLoop {
      mm: self.mm.clone(),
      connection_manager: self.connection_manager.clone(),
      event_rx: event_receiver,
    };
    let join_handle = tokio::spawn(run_loop.run_loop());
    Ok(join_handle)
  }

  /// 检查 Agent 健康状态（现在主要关注任务清理）
  pub async fn check_agent_health(&self) -> Result<(), DataError> {
    debug!("Checking agent health and cleaning up zombie tasks");

    // 清理僵尸任务
    self.cleanup_zombie_tasks().await
  }

  /// 清理僵尸任务
  async fn cleanup_zombie_tasks(&self) -> Result<(), DataError> {
    debug!("Cleaning up zombie tasks");

    // 查找运行时间过长的任务实例
    let zombie_instances = TaskInstanceBmc::find_zombie_instances(&self.mm).await?;

    for instance_ref in zombie_instances.iter() {
      warn!("Found zombie task instance: {}", instance_ref.id);

      // 通过 AgentRegistry 检查对应的 Agent 是否还在线
      let is_online = self.connection_manager.is_agent_online(&instance_ref.agent_id)?;

      if !is_online {
        // Agent 离线，取消任务
        self.cancel_zombie_task(instance_ref).await?;
      } else {
        // Agent 在线但任务可能卡住，发送取消命令
        self.request_task_cancellation(instance_ref).await?;
      }
    }

    if !zombie_instances.is_empty() {
      info!("Cleaned up {} zombie tasks", zombie_instances.len());
    }

    Ok(())
  }

  /// 取消僵尸任务
  async fn cancel_zombie_task(&self, instance: &TaskInstanceEntity) -> Result<(), DataError> {
    info!("Cancelling zombie task instance: {}", instance.id);

    // 更新任务实例状态
    let instance_update = TaskInstanceForUpdate {
      status: Some(TaskInstanceStatus::Failed),
      completed_at: Some(now_offset()),
      error_message: Some("Task became zombie (cleanup)".to_string()),
      ..Default::default()
    };

    TaskInstanceBmc::update_by_id(&self.mm, instance.id, instance_update).await?;

    // 更新任务状态
    let task_update = TaskForUpdate { status: Some(TaskStatus::Failed), ..Default::default() };

    TaskBmc::update_by_id(&self.mm, instance.task_id, task_update).await?;

    Ok(())
  }

  /// 请求任务取消
  async fn request_task_cancellation(&self, instance: &TaskInstanceEntity) -> Result<(), DataError> {
    info!("Requesting cancellation for task instance: {}", instance.id);

    // TODO: 发送取消命令给 Agent
    // 这里应该通过消息队列或 gRPC 发送取消命令

    // 标记任务为取消中
    let instance_update = TaskInstanceForUpdate { status: Some(TaskInstanceStatus::Cancelled), ..Default::default() };

    TaskInstanceBmc::update_by_id(&self.mm, instance.id, instance_update).await?;

    Ok(())
  }

  /// 获取在线 Agent 列表（通过 AgentRegistry）
  pub async fn get_agents(&self, server_ids: &[Uuid]) -> Result<Vec<AgentEntity>, DataError> {
    let online_agents = self.connection_manager.get_online_agents()?;
    if online_agents.is_empty() {
      return Ok(vec![]);
    }

    let online_agent_ids: Vec<Uuid> = online_agents.into_iter().map(|agent| agent.agent_id).collect();

    let agent_svc = AgentSvc::new(self.mm.clone());

    let filter = AgentFilter {
      id: Some(OpValsUuid::in_(online_agent_ids)),
      server_id: if server_ids.is_empty() { None } else { Some(OpValsUuid::in_(server_ids.to_vec())) },
      status: Some(OpValsInt32::eq(AgentStatus::Online as i32)),
      ..Default::default()
    };
    let agents = agent_svc.find_many(filter, None).await?;
    Ok(agents)
  }

  /// 获取 Agent 详细信息（包含可靠性统计）
  pub async fn get_agent_details(&self, agent_id: &Uuid) -> Result<Option<serde_json::Value>, DataError> {
    // 通过 AgentRegistry 获取基础信息
    if let Some(agent) = self.connection_manager.get_agent(agent_id)? {
      // 获取可靠性统计
      let details = serde_json::to_value(&agent)?;
      Ok(Some(details))
    } else {
      Ok(None)
    }
  }

  /// 选择最佳 Agent（基于可靠性统计和负载均衡）
  pub async fn select_best_agent(
    &self,
    _task_requirements: Option<serde_json::Value>,
  ) -> Result<Option<Uuid>, DataError> {
    let online_agents = self.connection_manager.get_online_agents()?;

    if online_agents.is_empty() {
      return Ok(None);
    }

    // 简单的负载均衡：选择连续失败次数最少、任务数最少的 Agent
    let best_agent = online_agents
      .iter()
      .min_by_key(|agent| {
        let stats = agent.stats();
        (stats.consecutive_failures, stats.total_tasks)
      })
      .cloned();

    Ok(best_agent.map(|agent| agent.agent_id))
  }

  /// 更新任务执行统计
  pub async fn update_task_stats(
    &self,
    agent_id: &Uuid,
    success: bool,
    response_time_ms: f64,
  ) -> Result<(), DataError> {
    if let Some(agent) = self.connection_manager.get_agent(agent_id)? {
      agent.update_stats(success, response_time_ms);
    }

    Ok(())
  }

  /// 刷新 Agent 状态（现在主要从 AgentRegistry 获取）
  pub async fn refresh_agent_status(&self) -> Result<(), DataError> {
    debug!("Refreshing agent status from AgentRegistry");

    // 获取当前在线 Agent 数量和列表，用于统计信息更新
    let online_count = self.connection_manager.get_online_count()?;
    let online_agents = self.connection_manager.get_online_agents()?;
    info!("Current online agents: {}, agent list: {:?}", online_count, online_agents);

    // TODO: 待实现

    Ok(())
  }

  /// 获取统计信息（包含可靠性数据）
  pub async fn get_stats(&self) -> Result<serde_json::Value, DataError> {
    let online_agents = self.connection_manager.get_online_agents()?;
    let online_count = online_agents.len();

    let mut total_tasks: u64 = 0;
    let mut total_successes: u64 = 0;
    let mut agents_with_failures: u64 = 0;

    for agent in online_agents {
      let stats = agent.stats();
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
  event_rx: mpsc::UnboundedReceiver<AgentEvent>,
}

impl AgentEventRunLoop {
  /// 处理 Agent 事件
  async fn run_loop(mut self) {
    let agent_svc = AgentSvc::new(self.mm.clone());
    while let Some(event) = self.event_rx.recv().await {
      match event {
        AgentEvent::Heartbeat { agent_id, .. } => {
          if let Err(e) = agent_svc.update_agent_heartbeat(&agent_id).await {
            error!("Failed to update agent heartbeat agent {}: {:?}", agent_id, e);
          }
        }
        AgentEvent::TaskInstanceChanged { agent_id, payload } => {
          if let Err(e) = self.process_task_instance_changed(agent_id, payload).await {
            error!("Failed to process task instance changed agent {}: {:?}", agent_id, e);
          }
        }
        AgentEvent::TaskPollRequest { agent_id, request } => {
          if let Err(e) = self.process_task_poll(agent_id, request).await {
            error!("Failed to process task poll request agent {}: {:?}", agent_id, e);
          }
        }
        AgentEvent::Registered { agent_id, payload } => match agent_svc.handle_register(&agent_id, &payload).await {
          Ok(response) => {
            let message = WebSocketCommand::new(CommandKind::AgentRegistered, response);
            if let Err(e) = self.connection_manager.send_to_agent(&agent_id, message) {
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
      }
    }
  }

  /// Agent poll task 时不对 Server 绑定的 Namespace 进行过滤，直接拉取符合要求的最紧急的 TaskInstanceEntity。按 request 条件进行过滤
  async fn process_task_poll(&self, agent_id: Uuid, request: Arc<TaskPollRequest>) -> Result<(), DataError> {
    info!("Agent {} task poll request: {:?}", agent_id, request);
    let task_instances = TaskInstanceBmc::find_many_by_poll(&self.mm, &request).await?;
    let task_map = TaskBmc::find_many(
      &self.mm,
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
            warn!("TaskEntity({}) of TaskInstanceEntity({}) not exists", task_id, task_instance.id);
            None
          }
        }
      })
      .collect::<Vec<_>>();

    // 向 Agent 发送 TaskPollResponse
    let parameters = serde_json::to_value(TaskPollResponse { tasks, has_more: false, next_poll_interval: 0 })?;
    let command = WebSocketCommand::new(CommandKind::DispatchTask, parameters);
    self.connection_manager.send_to_agent(&agent_id, command)?;

    Ok(())
  }

  async fn process_task_instance_changed(
    &self,
    agent_id: Uuid,
    payload: Arc<TaskInstanceUpdated>,
  ) -> Result<(), DataError> {
    todo!()
  }
}
