use fusion_scheduler_api::v1::CreateJobRequest;
use modql::field::Fields;
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::DbRowType;
use uuid::Uuid;

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedJob {
  pub id: Uuid,
  pub r#type: i32,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<serde_json::Value>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedJob {}

#[derive(Debug, Fields)]
pub struct SchedJobForCreate {
  pub id: Option<Uuid>,
  #[field(name = "type")]
  pub r#type: i32,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<serde_json::Value>,
}

impl From<CreateJobRequest> for SchedJobForCreate {
  fn from(job: CreateJobRequest) -> Self {
    Self {
      id: None,
      r#type: job.job_type,
      description: job.description,
      tags: job.tags,
      data: job.data.map(|bytes| serde_json::to_value(bytes).unwrap()),
    }
  }
}

#[derive(Debug, Fields)]
pub struct SchedJobForUpdate {
  pub r#type: Option<i32>,
  pub description: Option<String>,
  pub tags: Option<Vec<String>>,
  pub data: Option<Vec<u8>>,
}
