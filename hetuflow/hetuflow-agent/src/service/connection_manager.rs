use ultimate_common::time::OffsetDateTime;
use uuid::Uuid;

/// WebSocket 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
  /// 断开连接
  Disconnected,
  /// 连接中
  Connecting,
  /// 已连接
  Connected,
  /// 已注册
  Registered,
  /// 连接错误
  Error(String),
}

/// 连接管理器配置
#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
  /// WebSocket 服务器 URL
  pub server_url: String,
  /// Agent ID
  pub agent_id: Uuid,
  /// Agent 名称
  pub agent_name: String,
  /// Agent 标签
  pub agent_tags: Vec<String>,
  /// 心跳间隔（秒）
  pub heartbeat_interval_seconds: u64,
  /// 连接超时（秒）
  pub connection_timeout_seconds: u64,
  /// 重连间隔（秒）
  pub reconnect_interval_seconds: u64,
  /// 最大重连次数
  pub max_reconnect_attempts: u32,
  /// 消息队列大小
  pub message_queue_size: usize,
}

impl Default for ConnectionManagerConfig {
  fn default() -> Self {
    Self {
      server_url: "ws://localhost:8080/ws/agent".to_string(),
      agent_id: Uuid::new_v4(),
      agent_name: "hetuflow-agent".to_string(),
      agent_tags: vec![],
      heartbeat_interval_seconds: 30,
      connection_timeout_seconds: 10,
      reconnect_interval_seconds: 5,
      max_reconnect_attempts: 10,
      message_queue_size: 1000,
    }
  }
}

/// 连接统计信息
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
  /// 连接次数
  pub connection_count: u64,
  /// 重连次数
  pub reconnection_count: u64,
  /// 发送消息数
  pub messages_sent: u64,
  /// 接收消息数
  pub messages_received: u64,
  /// 心跳发送次数
  pub heartbeats_sent: u64,
  /// 最后心跳时间
  pub last_heartbeat_at: Option<OffsetDateTime>,
  /// 连接建立时间
  pub connected_at: Option<OffsetDateTime>,
  /// 最后错误
  pub last_error: Option<String>,
}

/// 连接管理器
/// 负责与 HetuFlow Gateway 的连接管理、心跳机制和消息传输
#[derive(Debug)]
pub struct ConnectionManager {}

impl ConnectionManager {
  /// 创建新的连接管理器
  pub fn new() -> Self {
    Self {}
  }
}
