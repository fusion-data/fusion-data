use std::sync::Arc;

use hetuflow_core::protocol::{AgentRegisterRequest, TaskInstanceUpdated};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{gateway::GatewayError, model::AgentConnection};

/// Agent 事件类型 - 统一的运行态事件
/// 网关事件。此为 Server 内部使用事件（基于 Agent 发送过来的消息包装），非 Agent 直接发送到 Server 的事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
  /// Agent 连接建立
  Connected { agent_id: Uuid, remote_addr: String },
  /// Agent 注册
  Registered { agent_id: Uuid, payload: Arc<AgentRegisterRequest> },
  /// Agent 取消注册
  Unregistered { agent_id: Uuid, reason: String },
  /// Agent 心跳更新
  Heartbeat { agent_id: Uuid, timestamp: i64 },
  /// Agent 任务实例状态变更
  TaskInstanceChanged { agent_id: Uuid, payload: Arc<TaskInstanceUpdated> },
}

/// Agent 注册表接口 - Agent 运行态的唯一抽象
///
/// 此 trait 作为"Agent 运行态"的单一事实来源(Single Source of Truth)，
/// 提供统一的在线/离线状态管理、心跳更新和事件订阅功能。
#[async_trait::async_trait]
pub trait AgentRegistry: Send + Sync {
  /// 获取在线 Agent 列表
  ///
  /// 返回当前所有在线的 Agent ID 列表
  async fn get_online_agents(&self) -> Result<Vec<Arc<AgentConnection>>, GatewayError>;

  /// 查找指定 Agent 信息
  ///
  /// # 参数
  /// * `agent_id` - Agent ID
  ///
  /// # 返回
  /// 如果 Agent 存在则返回其信息，否则返回 None
  async fn get_agent(&self, agent_id: &Uuid) -> Result<Option<Arc<AgentConnection>>, GatewayError>;

  /// 获取在线 Agent
  ///
  /// # 参数
  /// * `agent_id` - Agent ID
  ///
  /// # 返回
  /// 如果 Agent 存在则返回其信息，否则返回 Err(ConnectionNotFound)
  async fn find_online_agent(&self, agent_id: &Uuid) -> Result<Arc<AgentConnection>, GatewayError>;

  /// 订阅 Agent 事件
  ///
  /// # 参数
  /// * `handler` - 事件处理器发送端
  ///
  /// # 返回
  /// 事件接收端，用于监听 Agent 状态变化事件
  async fn subscribe_events(&self, handler: mpsc::UnboundedSender<AgentEvent>) -> Result<(), GatewayError>;

  /// 获取在线 Agent 数量
  async fn get_online_count(&self) -> Result<u32, GatewayError>;

  /// 检查指定 Agent 是否在线
  ///
  /// # 参数
  /// * `agent_id` - Agent ID
  ///
  /// # 返回
  /// 如果 Agent 在线返回 true，否则返回 false
  async fn is_agent_online(&self, agent_id: &Uuid) -> Result<bool, GatewayError>;
}

/// 类型别名，用于方便使用
pub type AgentRegistryRef = Arc<dyn AgentRegistry>;
