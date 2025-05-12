use modelsql::field::Fields;
use modelsql::DbRowType;
use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use uuid::Uuid;

#[derive(Debug, Default, FromRow, Fields)]
pub struct Policy {
  pub id: Uuid,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: i32,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for Policy {}

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
