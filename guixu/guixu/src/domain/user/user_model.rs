use modelsql::field::Fields;
use sea_query::enum_def;
use serde::Serialize;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow, Fields)]
#[enum_def(table_name = "user_entity")]
pub struct UserEntity {
  pub id: Uuid,
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub password: Option<String>,
  pub personalization_answers: Option<serde_json::Value>,
  pub settings: Option<serde_json::Value>,
  pub status: i32,
  pub mfa_enabled: bool,
  pub mfa_secret: Option<String>,
  pub mfa_recovery_codes: Option<String>,
  pub role: String,
  pub ctime: UtcDateTime,
  pub cid: i64,
  pub utime: Option<UtcDateTime>,
  pub uid: Option<i64>,
}

/// 认证身份表
#[derive(Debug, Clone, Serialize, FromRow, Fields)]
#[enum_def(table_name = "auth_identity")]
pub struct AuthIdentity {
  pub user_id: Uuid,
  pub provider_id: String,
  pub provider_kind: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
}

/// 用户API密钥表
#[derive(Debug, Clone, Serialize, FromRow, Fields)]
#[enum_def(table_name = "user_api_keys")]
pub struct UserApiKeys {
  pub id: String,
  pub user_id: Uuid,
  pub label: String,
  pub api_key: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub scopes: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_user_model() {
    assert_eq!(UserEntityIden::Table.as_ref(), "user_entity");
    assert_eq!(AuthIdentityIden::Table.as_ref(), "auth_identity");
    assert_eq!(UserApiKeysIden::Table.as_ref(), "user_api_keys");
  }
}
