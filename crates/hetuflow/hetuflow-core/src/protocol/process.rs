use fusion_common::time::now_epoch_millis;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 进程信息
#[derive(Clone, Serialize)]
pub struct ProcessInfo {
  /// 进程ID
  pub pid: u32,
  /// 任务实例ID
  pub instance_id: Uuid,
  /// 进程状态
  pub status: ProcessStatus,
  /// 启动时间
  pub started_at: i64,
  /// 完成时间
  pub completed_at: Option<i64>,
  /// 退出码
  pub exit_code: Option<i32>,
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
  pub timestamp: i64,
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
  /// 任务实例ID
  pub instance_id: Uuid,
  /// 事件类型
  pub kind: ProcessEventKind,
  /// 事件时间
  pub timestamp: i64,
  /// 事件数据
  pub data: Option<String>,
}

impl ProcessEvent {
  pub fn new(instance_id: Uuid, kind: ProcessEventKind) -> Self {
    Self::new_with_data(instance_id, kind, None)
  }

  pub fn new_with_data(instance_id: Uuid, kind: ProcessEventKind, data: Option<String>) -> Self {
    Self { instance_id, kind, timestamp: now_epoch_millis(), data }
  }

  pub fn with_data(mut self, data: String) -> Self {
    self.data = Some(data);
    self
  }

  pub fn with_timestamp(mut self, timestamp: i64) -> Self {
    self.timestamp = timestamp;
    self
  }
}

/// 进程事件类型
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProcessEventKind {
  /// 进程启动
  Started,
  /// 进程运行结束退出
  Exited,
  /// 进程收到 SIGTERM 信号退出
  Sigterm,
  /// 进程收到 SIGKILL 信号初杀死
  Sigkill,
  /// 资源违规
  ResourceViolation,
  /// 进程变为僵尸
  BecameZombie,
}
