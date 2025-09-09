use fusion_common::time::OffsetDateTime;
use modelsql::{field::Fields, generate_enum_i32_to_sea_query_value};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, sqlx::Type)]
#[repr(i32)]
pub enum AuthProviderKind {
  Wechat,
  Alipay,
  Weibo,
  QQ,
  OSChina,
  Baidu,
  Gitee,
  Github,
}
generate_enum_i32_to_sea_query_value!(Enum: AuthProviderKind,);

/// 认证提供者同步历史表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "auth_provider_sync_history")]
pub struct AuthProviderSyncHistoryEntity {
  pub id: Uuid,
  pub provider_kind: AuthProviderKind,
  pub run_mode: String,
  pub status: String,
  pub started_at: OffsetDateTime,
  pub ended_at: Option<OffsetDateTime>,
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
  pub expires_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_models() {
    assert_eq!(AuthProviderSyncHistoryEntityIden::Table.as_ref(), "auth_provider_sync_history");
    assert_eq!(InvalidAuthTokenIden::Table.as_ref(), "invalid_auth_token");
  }
}
