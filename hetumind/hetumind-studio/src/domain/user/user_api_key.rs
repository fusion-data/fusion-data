use modelsql::{field::Fields, postgres::PgRowType};
use sea_query::enum_def;
use serde::Serialize;
use sqlx::FromRow;
use fusion_common::time::OffsetDateTime;

/// 用户API密钥表
#[derive(Debug, Clone, Serialize, FromRow, Fields)]
#[enum_def(table_name = "user_api_key")]
pub struct UserApiKey {
  pub id: String,
  pub user_id: i64,
  pub label: String,
  pub api_key: String,
  pub scopes: Option<serde_json::Value>,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
  pub updated_at: Option<OffsetDateTime>,
  pub updated_by: Option<i64>,
}
impl PgRowType for UserApiKey {}
