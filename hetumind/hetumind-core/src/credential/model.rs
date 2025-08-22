use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 凭证唯一标识符
pub type CredentialId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInfo {
  pub id: CredentialId,
  pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
  Bearer,
}
