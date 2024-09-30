use fusion_scheduler_api::v1::CreateJobRequest;
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64, OpValsString, OpValsValue},
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate::DataError;
use ultimate_api::v1::{Page, PagePayload, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::{
  datetime_to_sea_value, try_into_op_vals_int32_opt, try_into_op_vals_value_opt_with_filter_int64,
  try_into_op_vals_value_opt_with_filter_string, uuid_to_sea_value, DbRowType,
};
use uuid::Uuid;

use crate::pb::fusion_scheduler::v1::{JobFilterRequest, PageJobRequest, PageJobResponse, SchedJobDto};

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct SchedJob {
  pub id: Uuid,
  pub r#type: i32,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<Vec<u8>>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedJob {}

impl From<SchedJob> for SchedJobDto {
  fn from(row: SchedJob) -> Self {
    SchedJobDto {
      id: row.id.to_string(),
      r#type: row.r#type,
      description: row.description,
      tags: row.tags,
      data: row.data,
      cid: row.cid,
      ctime: row.ctime.timestamp_millis(),
      mid: row.mid,
      mtime: row.mtime.map(|v| v.timestamp_millis()),
    }
  }
}

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

#[derive(Debug, Default, FilterNodes)]
pub struct SchedJobFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub id: Option<OpValsValue>,

  pub r#type: Option<OpValsInt32>,

  pub status: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

impl TryFrom<JobFilterRequest> for SchedJobFilter {
  type Error = DataError;
  fn try_from(value: JobFilterRequest) -> Result<Self, Self::Error> {
    let f = Self {
      id: try_into_op_vals_value_opt_with_filter_string(value.id)?,
      r#type: try_into_op_vals_int32_opt(value.r#type)?,
      status: try_into_op_vals_int32_opt(value.status)?,
      ctime: try_into_op_vals_value_opt_with_filter_int64(value.ctime)?,
      mtime: try_into_op_vals_value_opt_with_filter_int64(value.mtime)?,
    };
    Ok(f)
  }
}

pub struct SchedJobForPage {
  pub pagination: Pagination,
  pub filter: Vec<SchedJobFilter>,
}

impl TryFrom<PageJobRequest> for SchedJobForPage {
  type Error = DataError;

  fn try_from(value: PageJobRequest) -> Result<Self, DataError> {
    let mut filter = Vec::with_capacity(value.filter.len());
    for f in value.filter {
      filter.push(f.try_into()?);
    }
    let f = Self { pagination: value.pagination.unwrap_or_default(), filter };
    Ok(f)
  }
}

impl From<PagePayload<SchedJob>> for PageJobResponse {
  fn from(value: PagePayload<SchedJob>) -> Self {
    Self { page: Some(value.page), items: value.items.into_iter().map(Into::into).collect() }
  }
}
