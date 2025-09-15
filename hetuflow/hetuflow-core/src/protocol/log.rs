use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 日志类型枚举
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogType {
  /// 标准输出
  Stdout,
  /// 标准错误
  Stderr,
  /// 系统日志
  System,
  /// 调试日志
  Debug,
  /// 信息日志
  Info,
  /// 警告日志
  Warn,
  /// 错误日志
  Error,
}

/// 日志消息协议结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogMessage {
  /// 任务ID
  pub task_id: Uuid,
  /// 任务实例ID
  pub task_instance_id: Uuid,
  /// 日志序列号，用于保证顺序
  pub sequence: u64,
  /// 日志类型
  pub log_type: LogType,
  /// 日志内容
  pub content: String,
  /// 时间戳（毫秒）
  pub timestamp: i64,
  /// Agent ID
  pub agent_id: String,
  /// 进程ID（可选）
  pub process_id: Option<u32>,
  /// 日志级别（可选，用于结构化日志）
  pub level: Option<String>,
  /// 日志来源（可选，如文件名、模块名等）
  pub source: Option<String>,
}

impl LogMessage {
  /// 创建新的日志消息
  pub fn new(
    task_id: Uuid,
    task_instance_id: Uuid,
    sequence: u64,
    log_type: LogType,
    content: String,
    timestamp: i64,
    agent_id: String,
  ) -> Self {
    Self {
      task_id,
      task_instance_id,
      sequence,
      log_type,
      content,
      timestamp,
      agent_id,
      process_id: None,
      level: None,
      source: None,
    }
  }

  /// 设置进程ID
  pub fn with_process_id(mut self, process_id: u32) -> Self {
    self.process_id = Some(process_id);
    self
  }

  /// 设置日志级别
  pub fn with_level(mut self, level: String) -> Self {
    self.level = Some(level);
    self
  }

  /// 设置日志来源
  pub fn with_source(mut self, source: String) -> Self {
    self.source = Some(source);
    self
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_log_message_creation() {
    let task_id = Uuid::now_v7();
    let task_instance_id = Uuid::now_v7();
    let log_msg = LogMessage::new(
      task_id,
      task_instance_id,
      1,
      LogType::Stdout,
      "Test log message".to_string(),
      1234567890,
      "agent-001".to_string(),
    )
    .with_process_id(12345)
    .with_level("INFO".to_string())
    .with_source("test.rs".to_string());

    assert_eq!(log_msg.task_id, task_id);
    assert_eq!(log_msg.task_instance_id, task_instance_id);
    assert_eq!(log_msg.sequence, 1);
    assert_eq!(log_msg.log_type, LogType::Stdout);
    assert_eq!(log_msg.content, "Test log message");
    assert_eq!(log_msg.process_id, Some(12345));
    assert_eq!(log_msg.level, Some("INFO".to_string()));
    assert_eq!(log_msg.source, Some("test.rs".to_string()));
  }

  #[test]
  fn test_log_batch_creation() {
    let task_id = Uuid::now_v7();
    let task_instance_id = Uuid::now_v7();
    let log_msg = LogMessage::new(
      task_id,
      task_instance_id,
      1,
      LogType::Stderr,
      "Error message".to_string(),
      1234567890,
      "agent-001".to_string(),
    );

    let batch = LogBatch::new(vec![log_msg]).with_compression(true);

    assert_eq!(batch.messages.len(), 1);
    assert_eq!(batch.compressed, true);
    assert!(!batch.batch_id.is_nil());
  }

  #[test]
  fn test_log_type_serialization() {
    let log_types = vec![
      LogType::Stdout,
      LogType::Stderr,
      LogType::System,
      LogType::Debug,
      LogType::Info,
      LogType::Warn,
      LogType::Error,
    ];

    for log_type in log_types {
      let serialized = serde_json::to_string(&log_type).unwrap();
      let deserialized: LogType = serde_json::from_str(&serialized).unwrap();
      assert_eq!(log_type, deserialized);
    }
  }
}