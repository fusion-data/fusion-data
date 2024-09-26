use modql::field::Fields;
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::DbRowType;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Fields)]
#[enum_def]
pub struct SchedTrigger {
  pub id: Uuid,
  pub r#type: i32,
  pub schedule: serde_json::Value,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub data: Vec<u8>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedTrigger {}
