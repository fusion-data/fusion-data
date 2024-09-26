use fusion_scheduler_api::v1::JobDefinition;
use modql::field::Fields;
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::DbRowType;
use uuid::Uuid;

use crate::pb::fusion_scheduler::v1::CreateJobRequest;

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedJob {
  pub id: Uuid,
  pub r#type: i32,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub data: Vec<u8>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedJob {}

#[derive(Debug, Fields)]
pub struct SchedJobForCreate {
  pub id: Option<Uuid>,
  pub r#type: i32,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub data: Vec<u8>,
}

impl From<CreateJobRequest> for SchedJobForCreate {
  fn from(req: CreateJobRequest) -> Self {
    Self { id: None, r#type: req.r#type, description: req.description, tags: req.tags, data: req.data }
  }
}

impl From<JobDefinition> for SchedJobForCreate {
  fn from(job: JobDefinition) -> Self {
    let id = if job.job_id.is_empty() { None } else { job.job_id.parse().ok() };
    Self { id, r#type: job.job_type, description: job.description, tags: job.tags, data: job.data }
  }
}

#[derive(Debug, Fields)]
pub struct SchedJobForUpdate {
  pub r#type: Option<i32>,
  pub description: Option<String>,
  pub tags: Option<Vec<String>>,
  pub data: Option<Vec<u8>>,
}
