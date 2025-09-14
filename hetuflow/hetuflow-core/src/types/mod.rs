#[cfg(feature = "with-cli")]
mod cli;

use std::sync::Arc;

use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::AsRefStr;
use uuid::Uuid;

use crate::protocol::{AcquireTaskResponse, AgentRegisterResponse};

/// 作业类型 (ScheduleKind) - 定义了 Job 的核心调度和行为模式
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum ScheduleKind {
  /// Cron 定时作业
  Cron = 1,
  /// 间隔定时作业。可以通过设置最大执行次数为 1 次来表达 Once 执行，可以通过设置 start_time 来设置定时执行时间
  Interval = 2,
  /// 守护进程作业
  Daemon = 3,
  /// 事件驱动作业
  Event = 4,
  /// 流程任务
  Flow = 5,
}

/// 作业状态
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum JobStatus {
  /// 已创建
  Created = 1,
  /// 已禁用
  Disabled = 99,
  /// 已启用
  Enabled = 100,
}

/// 调度状态
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum ScheduleStatus {
  /// 已创建
  Created = 1,
  /// 调度已过期，不再生成有效
  Expired = 98,
  /// 已禁用
  Disabled = 99,
  /// 已启用
  Enabled = 100,
}

/// 任务状态
///
/// 任务状态状态机：
///
/// ```mermaid
/// stateDiagram-v2
///   [*] --> Pending
///   Pending --> Locked : Server从数据库获取数据时
///   Locked --> Dispatched : Agent已获取任务
///   Dispatched --> Running : Agent已运行任务
///   Running --> Failed : 任务执行失败
///   Failed --> WaitingRetry : Agent执行失败，等待重试（未达到重试次数限制）
///   WaitingRetry --> Running : Agent重试执行
///   Running --> Cancelled
///   Running --> Succeeded
///   Failed --> [*] : 任务失败结束（达到重试次数限制）
///   Cancelled --> [*] : 任务取消结束
///   Succeeded --> [*] : 任务成功结束
/// ```
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum TaskStatus {
  /// 等待分发
  Pending = 1,
  /// 已锁定，等待分发到 agent 执行
  Locked = 10,
  /// 已分发到 agent
  Dispatched = 20,
  /// 等待重试
  WaitingRetry = 30,
  /// 错误
  Failed = 90,
  /// 取消完成
  Cancelled = 99,
  /// 成功完成
  Succeeded = 100,
}

/// 任务执行状态
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum TaskInstanceStatus {
  /// 等待执行
  Pending = 1,
  /// 已分发
  Dispatched = 5,
  /// 执行中
  Running = 10,
  /// 执行超时
  Timeout = 20,
  /// 已暂停
  Paused = 30,
  /// 已跳过
  Skipped = 40,
  /// 执行失败
  Failed = 90,
  /// 已取消
  Cancelled = 99,
  /// 执行成功
  Succeeded = 100,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum ServerStatus {
  /// 不活跃
  Inactive = 99,
  /// 从节点
  Active = 100,
}

/// Agent 状态
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum AgentStatus {
  Idle = 10,          // 空闲
  Busy = 20,          // 忙碌
  Connecting = 30,    // 连接中
  Disconnecting = 31, // 断开连接中
  Offline = 90,       // 离线
  Error = 99,         // 错误状态
  Online = 100,       // 在线
}

/// 从 Server 发向 Agent 的命令类型
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum CommandKind {
  Shutdown = 1,        // 关闭指令
  UpdateConfig = 2,    // 更新配置
  ClearCache = 3,      // 清理缓存
  FetchMetrics = 4,    // 更新指标
  AgentRegistered = 5, // Agent 注册成功
  DispatchTask = 6,    // 分发任务
  CancelTask = 7,      // 取消任务
  LogForward = 8,      // 日志转发
}

#[derive(Clone)]
pub enum HetuflowCommand {
  Shutdown,
  UpdateConfig,
  ClearCache,
  FetchMetrics,
  AgentRegistered(Arc<AgentRegisterResponse>),
  AcquiredTask(Arc<AcquireTaskResponse>),
  /// TaskInstanceId
  CancelTask(Arc<Uuid>),
}

/// 任务控制类型
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum TaskControlKind {
  Stop = 1,    // 停止任务
  Pause = 2,   // 暂停任务
  Resume = 3,  // 恢复任务
  Restart = 4, // 重启任务
  Skip = 5,    // 跳过任务
  Kill = 9,    // 强制终止任务
}

/// WebSocket 消息类型枚举
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum EventKind {
  /// 确认消息
  Ack = 1,
  /// 未确认消息
  Nack = 2,

  /// Agent 注册
  AgentRegister = 3,
  /// Agent 心跳
  AgentHeartbeat = 4,

  /// Agent 请求 AgentRequest <-> GatewayResponse
  PollTaskRequest = 5,

  /// Agent 事件 AgentEvent
  TaskChangedEvent = 6,
  /// 任务日志事件
  TaskLog = 7,
}

#[cfg(feature = "with-db")]
modelsql::generate_enum_i32_to_sea_query_value!(
  Enum: ScheduleKind,
  Enum: JobStatus,
  Enum: ScheduleStatus,
  Enum: TaskStatus,
  Enum: TaskInstanceStatus,
  Enum: ServerStatus,
  Enum: AgentStatus,
  Enum: CommandKind,
  Enum: TaskControlKind,
);

/// 资源限制配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ResourceLimits {
  /// 最大内存使用量 (MB)
  pub max_memory_mb: Option<u64>,
  /// 最大CPU使用率 (0.0-1.0)
  pub max_cpu_percent: Option<f64>,
  /// 最大执行时间 (秒)
  pub max_execution_time_secs: Option<u64>,
  /// 最大输出大小 (字节)
  pub max_output_size_bytes: Option<u64>,
}

impl Default for ResourceLimits {
  fn default() -> Self {
    Self {
      max_memory_mb: Some(1024),                     // 默认1GB内存限制
      max_cpu_percent: Some(0.8),                    // 默认80%CPU限制
      max_execution_time_secs: Some(3600),           // 默认1小时执行时间限制
      max_output_size_bytes: Some(10 * 1024 * 1024), // 默认10MB输出限制
    }
  }
}
