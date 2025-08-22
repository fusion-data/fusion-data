mod agent;
mod heartbeat;
mod task;
mod websocket;

pub use agent::*;
pub use heartbeat::*;
pub use task::*;
pub use websocket::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 文件上传请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileUploadRequest {
  pub task_id: Uuid,                     // 关联任务ID
  pub file_name: String,                 // 文件名
  pub file_size: u64,                    // 文件大小
  pub content_hash: String,              // 内容哈希
  pub metadata: HashMap<String, String>, // 文件元数据
}

/// 文件下载请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileDownloadRequest {
  pub task_id: Uuid,           // 关联任务ID
  pub file_path: String,       // 文件路径
  pub version: Option<String>, // 文件版本
}

/// 错误响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
  /// 错误码
  pub err_code: i32,
  /// 错误消息
  pub err_msg: String,
  /// 详细信息
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
  /// 时间戳
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<i64>,
}

/// 确认消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AckMessage {
  /// 原始消息ID
  pub message_id: Uuid,
  /// 处理状态
  pub status: String,
  /// 详细信息
  pub details: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayCommand {
  Send { command: WebSocketCommand, agent_id: Uuid },
  Broadcast { command: WebSocketCommand },
}

#[derive(Debug)]
pub enum SchedulerEvent {
  TaskDispatched { task_id: Uuid, agent_id: Uuid },
  TaskCompleted { task_id: Uuid, agent_id: Uuid, success: bool },
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_error_response_serialization() {
    let error = ErrorResponse {
      err_code: 404,
      err_msg: "Not found".to_string(),
      details: Some(json!({"path": "/test"})),
      timestamp: Some(1234567890),
    };

    let serialized = serde_json::to_string(&error).unwrap();
    let deserialized: ErrorResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.err_code, 404);
    assert_eq!(deserialized.err_msg, "Not found");
    assert_eq!(deserialized.details, Some(json!({"path": "/test"})));
  }

  #[test]
  fn test_ack_message_serialization() {
    let ack =
      AckMessage { message_id: Uuid::new_v4(), status: "success".to_string(), details: Some("processed".to_string()) };

    let serialized = serde_json::to_string(&ack).unwrap();
    let deserialized: AckMessage = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.status, "success");
    assert_eq!(deserialized.details, Some("processed".to_string()));
  }

  #[test]
  fn test_file_upload_request_creation() {
    let task_id = Uuid::now_v7();
    let upload = FileUploadRequest {
      task_id,
      file_name: "test.txt".to_string(),
      file_size: 1024,
      content_hash: "abc123".to_string(),
      metadata: HashMap::from([("type".to_string(), "text".to_string())]),
    };

    assert_eq!(upload.task_id, task_id);
    assert_eq!(upload.file_name, "test.txt");
    assert_eq!(upload.file_size, 1024);
  }

  #[test]
  fn test_file_download_request_creation() {
    let task_id = Uuid::now_v7();
    let download =
      FileDownloadRequest { task_id, file_path: "/path/to/file.txt".to_string(), version: Some("v1.0".to_string()) };

    assert_eq!(download.task_id, task_id);
    assert_eq!(download.file_path, "/path/to/file.txt");
    assert_eq!(download.version, Some("v1.0".to_string()));
  }
}
