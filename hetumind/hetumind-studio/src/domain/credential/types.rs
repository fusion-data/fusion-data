use chrono::{DateTime, FixedOffset};
use hetumind_core::workflow::CredentialKind;
use serde::{Deserialize, Serialize};

use crate::domain::credential::CredentialEntity;

/// 凭证数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialData {
  /// 凭证原始数据
  pub data: String,
  /// 是否测试连接
  #[serde(default)]
  pub test_connection: bool,
}

/// 包含解密数据的凭证
#[derive(Debug, Clone, Serialize)]
pub struct CredentialWithDecryptedData {
  pub credential: CredentialEntity,
  pub decrypted_data: CredentialData,
}

/// 凭证验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialVerifyResult {
  pub success: bool,
  pub message: String,
  pub verify_time: DateTime<FixedOffset>,
}

/// 凭证验证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCredentialRequest {
  pub data: CredentialData,
  pub kind: CredentialKind,
}
