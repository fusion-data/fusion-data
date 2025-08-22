use serde::{Deserialize, Serialize};

/// 文件传输状态
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileTransferStatus {
  pub transfer_id: String,           // 传输ID
  pub file_name: String,             // 文件名
  pub file_size: u64,                // 文件大小
  pub bytes_transferred: u64,        // 已传输字节数
  pub status: String,                // 传输状态
  pub error_message: Option<String>, // 错误信息
}
