use fusion_common::time::OffsetDateTime;
use modelsql::field::Fields;
use modelsql::postgres::PgRowType;
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, FromRow, Fields)]
pub struct Policy {
  pub id: Uuid,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: i32,
  pub cid: i64,
  pub ctime: OffsetDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<OffsetDateTime>,
}
impl PgRowType for Policy {}

#[derive(Debug, Default, Fields)]
pub struct PolicyForCreate {
  pub id: Uuid,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: Option<i32>,
}

#[derive(Debug, Default, Fields)]
pub struct PolicyForUpdate {
  pub description: Option<String>,
  pub policy: Option<serde_json::Value>,
  pub status: Option<i32>,
}
