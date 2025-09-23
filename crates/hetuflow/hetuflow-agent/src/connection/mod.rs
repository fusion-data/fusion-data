mod command_runner;
mod connection_manager;
mod ws_runner;

pub use command_runner::*;
pub use connection_manager::*;
pub use ws_runner::*;

// /// WebSocket 连接状态
// #[derive(Debug, Clone, PartialEq)]
// pub enum ConnectionState {
//   /// 断开连接
//   Disconnected,
//   /// 连接中
//   Connecting,
//   /// 已连接
//   Connected,
//   /// 已注册
//   Registered,
//   /// 连接错误
//   Error(String),
// }

// /// 连接统计信息
// #[derive(Debug, Clone, Default)]
// pub struct ConnectionStats {
//   /// 连接次数
//   pub connection_count: u64,
//   /// 重连次数
//   pub reconnection_count: u64,
//   /// 发送消息数
//   pub messages_sent: u64,
//   /// 接收消息数
//   pub messages_received: u64,
//   /// 心跳发送次数
//   pub heartbeats_sent: u64,
//   /// 最后心跳时间
//   pub last_heartbeat_at: Option<OffsetDateTime>,
//   /// 连接建立时间
//   pub connected_at: Option<OffsetDateTime>,
//   /// 最后错误
//   pub last_error: Option<String>,
// }
