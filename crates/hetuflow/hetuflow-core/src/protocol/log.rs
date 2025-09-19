use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use uuid::Uuid;

/// 日志类型枚举
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LogKind {
  /// 标准输出
  Stdout,
  /// 标准错误
  Stderr,
}

/// 日志消息协议结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogMessage {
  /// 任务实例ID
  pub task_instance_id: Uuid,
  /// 日志序列号，用于保证顺序
  pub sequence: u32,
  /// 日志类型
  pub kind: LogKind,
  /// 日志内容
  pub content: String,
}

impl LogMessage {
  /// 创建新的日志消息
  pub fn new(task_instance_id: Uuid, sequence: u32, kind: LogKind, content: String) -> Self {
    Self { task_instance_id, sequence, kind, content }
  }
}

/// 日志批量消息，用于批量传输优化
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogBatch {
  /// 批次ID
  pub batch_id: Uuid,
  /// 日志消息列表
  pub messages: Vec<LogMessage>,
  /// 批次创建时间戳
  pub batch_timestamp: i64,
  /// 压缩标志
  pub compressed: bool,
}

impl LogBatch {
  /// 创建新的日志批次
  pub fn new(messages: Vec<LogMessage>) -> Self {
    Self {
      batch_id: Uuid::now_v7(),
      messages,
      batch_timestamp: fusion_common::time::now_epoch_millis(),
      compressed: false,
    }
  }

  /// 设置压缩标志
  pub fn with_compression(mut self, compressed: bool) -> Self {
    self.compressed = compressed;
    self
  }
}
