use fusion_common::time::OffsetDateTime;
use modelsql::{field::Fields, postgres::PgRowType};
use sea_query::enum_def;
use serde::Serialize;
use sqlx::FromRow;

/// 认证身份表
#[derive(Debug, Clone, Serialize, FromRow, Fields)]
#[enum_def(table_name = "auth_identity")]
pub struct AuthIdentity {
  pub user_id: i64,
  pub provider_id: String,
  pub provider_kind: String,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
  pub updated_at: Option<OffsetDateTime>,
  pub updated_by: Option<i64>,
}
impl PgRowType for AuthIdentity {}
