use std::sync::Arc;

use hetuflow_core::protocol::{
  AcquireTaskRequest, AgentLogMessage, HeartbeatEvent, RegisterAgentRequest, TaskInstanceChanged,
};

/// Agent 事件类型 - 统一的运行态事件
/// 网关事件。此为 Server 内部使用事件（基于 Agent 发送过来的消息包装），非 Agent 直接发送到 Server 的事件。
#[derive(Clone)]
pub enum AgentEvent {
  /// Agent 连接建立
  Connected { agent_id: String, remote_addr: Arc<String> },
  /// Agent 注册
  Registered { agent_id: String, payload: Arc<RegisterAgentRequest> },
  /// Task Log 上报
  TaskLog { agent_id: String, payload: Arc<AgentLogMessage> },
  /// Agent 心跳更新
  Heartbeat { agent_id: String, payload: Arc<HeartbeatEvent> },
  /// Agent 任务实例状态变更
  TaskInstanceChanged { agent_id: String, payload: Arc<TaskInstanceChanged> },
  /// Agent 任务轮询请求
  TaskPollRequest { agent_id: String, request: Arc<AcquireTaskRequest> },
  /// Agent 断开连接
  Unconnected { agent_id: String, reason: Arc<String> },
}

impl AgentEvent {
  pub fn new_register(agent_id: String, request: RegisterAgentRequest) -> Self {
    Self::Registered { agent_id, payload: Arc::new(request) }
  }

  pub fn new_heartbeat(agent_id: String, request: HeartbeatEvent) -> Self {
    Self::Heartbeat { agent_id, payload: Arc::new(request) }
  }

  pub fn new_task_poll_request(agent_id: String, request: AcquireTaskRequest) -> Self {
    Self::TaskPollRequest { agent_id, request: Arc::new(request) }
  }

  pub fn new_task_instance_changed(agent_id: String, request: TaskInstanceChanged) -> Self {
    Self::TaskInstanceChanged { agent_id, payload: Arc::new(request) }
  }

  pub fn new_task_log(agent_id: String, payload: AgentLogMessage) -> AgentEvent {
    Self::TaskLog { agent_id, payload: Arc::new(payload) }
  }
}
