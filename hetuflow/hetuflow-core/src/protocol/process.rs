use serde::{Deserialize, Serialize};
use ultimate_common::time::OffsetDateTime;
use uuid::Uuid;

/// 资源限制配置
#[derive(Serialize, Deserialize, Debug, Clone)]
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

/// 进程信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
  /// 进程ID
  pub pid: u32,
  /// 任务ID
  pub task_id: Uuid,
  /// 任务实例ID
  pub instance_id: Option<Uuid>,
  /// 进程状态
  pub status: ProcessStatus,
  /// 启动时间
  pub started_at: OffsetDateTime,
  /// 完成时间
  pub completed_at: Option<OffsetDateTime>,
  /// 退出码
  pub exit_code: Option<i32>,
  /// 资源使用情况
  pub resource_usage: Option<ResourceUsage>,
  /// 是否为守护进程
  pub is_daemon: bool,
}

/// 进程状态
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProcessStatus {
  /// 启动中
  Starting,
  /// 运行中
  Running,
  /// 已完成
  Completed,
  /// 已失败
  Failed,
  /// 被杀死
  Killed,
  /// 超时
  Timeout,
  /// 僵尸进程
  Zombie,
}

/// 资源使用情况
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceUsage {
  /// 内存使用量 (MB)
  pub memory_mb: f64,
  /// CPU使用率 (0.0-1.0)
  pub cpu_percent: f64,
  /// 运行时长 (秒)
  pub runtime_secs: u64,
  /// 输出大小 (字节)
  pub output_size_bytes: u64,
}

/// 资源违规信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceViolation {
  /// 违规类型
  pub violation_type: ResourceViolationType,
  /// 当前值
  pub current_value: f64,
  /// 限制值
  pub limit_value: f64,
  /// 违规时间
  pub timestamp: OffsetDateTime,
}

/// 资源违规类型
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ResourceViolationType {
  /// 内存超限
  MemoryExceeded,
  /// CPU超限
  CpuExceeded,
  /// 执行时间超限
  TimeoutExceeded,
  /// 输出大小超限
  OutputSizeExceeded,
}

/// 进程事件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessEvent {
  /// 进程ID
  pub pid: u32,
  /// 任务ID
  pub task_id: Uuid,
  /// 事件类型
  pub event_type: ProcessEventType,
  /// 事件时间
  pub timestamp: OffsetDateTime,
  /// 事件数据
  pub data: Option<serde_json::Value>,
}

/// 进程事件类型
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProcessEventType {
  /// 进程启动
  Started,
  /// 进程退出
  Exited,
  /// 进程被杀死
  Killed,
  /// 资源违规
  ResourceViolation,
  /// 进程变为僵尸
  BecameZombie,
}
