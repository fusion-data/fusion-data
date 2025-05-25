use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 认证提供者同步历史表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "auth_provider_sync_history")]
pub struct AuthProviderSyncHistory {
  pub id: i32,
  pub provider_kind: String,
  pub run_mode: String,
  pub status: String,
  pub started_at: UtcDateTime,
  pub ended_at: UtcDateTime,
  pub scanned: i32,
  pub created: i32,
  pub updated: i32,
  pub disabled: i32,
  pub error: Option<String>,
}

/// 无效认证令牌表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "invalid_auth_token")]
pub struct InvalidAuthToken {
  pub token: String,
  pub expires_at: UtcDateTime,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_models() {
    assert_eq!(AuthProviderSyncHistoryIden::Table.as_ref(), "auth_provider_sync_history");
    assert_eq!(InvalidAuthTokenIden::Table.as_ref(), "invalid_auth_token");
  }
}
