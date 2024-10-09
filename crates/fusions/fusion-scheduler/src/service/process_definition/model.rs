use fusion_scheduler_api::v1::CreateProcessDefinitionRequest;
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsValue},
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate::DataError;
use ultimate_api::v1::{PagePayload, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::{
  datetime_to_sea_value, try_into_op_vals_int32_opt, try_into_op_vals_value_opt_with_filter_int64,
  try_into_op_vals_value_opt_with_filter_string, uuid_to_sea_value, DbRowType,
};

use crate::pb::fusion_scheduler::v1::{
  PageProcessRequest, PageProcessResponse, ProcessFilterRequest, SchedProcessDto,
};

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct ProcessDefinition {
  pub id: i64,
  pub key: String,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<Vec<u8>>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for ProcessDefinition {}

impl From<ProcessDefinition> for SchedProcessDto {
  fn from(row: ProcessDefinition) -> Self {
    SchedProcessDto {
      id: row.id,
      key: row.key,
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
pub struct ProcessDefinitionForCreate {
  pub key: String,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<serde_json::Value>,
}

impl From<CreateProcessDefinitionRequest> for ProcessDefinitionForCreate {
  fn from(job: CreateProcessDefinitionRequest) -> Self {
    Self {
      key: job.process_key,
      description: job.description,
      tags: job.tags,
      data: job.data.map(|bytes| serde_json::to_value(bytes).unwrap()),
    }
  }
}

#[derive(Debug, Fields)]
pub struct ProcessDefinitionForUpdate {
  pub kind: Option<i32>,
  pub description: Option<String>,
  pub tags: Option<Vec<String>>,
  pub data: Option<Vec<u8>>,
}

#[derive(Debug, Default, FilterNodes)]
pub struct ProcessDefinitionFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub id: Option<OpValsValue>,

  pub status: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

impl TryFrom<ProcessFilterRequest> for ProcessDefinitionFilter {
  type Error = DataError;
  fn try_from(value: ProcessFilterRequest) -> Result<Self, Self::Error> {
    let f = Self {
      id: try_into_op_vals_value_opt_with_filter_string(value.id)?,
      status: try_into_op_vals_int32_opt(value.status)?,
      ctime: try_into_op_vals_value_opt_with_filter_int64(value.ctime)?,
      mtime: try_into_op_vals_value_opt_with_filter_int64(value.mtime)?,
    };
    Ok(f)
  }
}

pub struct SchedProcessForPage {
  pub pagination: Pagination,
  pub filter: Vec<ProcessDefinitionFilter>,
}

impl TryFrom<PageProcessRequest> for SchedProcessForPage {
  type Error = DataError;

  fn try_from(value: PageProcessRequest) -> Result<Self, DataError> {
    let mut filter = Vec::with_capacity(value.filter.len());
    for f in value.filter {
      filter.push(f.try_into()?);
    }
    let f = Self { pagination: value.pagination.unwrap_or_default(), filter };
    Ok(f)
  }
}

impl From<PagePayload<ProcessDefinition>> for PageProcessResponse {
  fn from(value: PagePayload<ProcessDefinition>) -> Self {
    Self { page: Some(value.page), items: value.items.into_iter().map(Into::into).collect() }
  }
}
