use fusion_flow_api::v1::{self, CreateProcessRequest, process_definition::ProcessStatus};
use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsUuid, OpValsValue},
  page::PageResult,
  postgres::PgRowType,
  utils::datetime_to_sea_value,
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_api::v1::{Page, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_core::DataError;
use ultimate_db::{
  try_into_op_vals_int32_opt, try_into_op_vals_uuid_with_filter_string, try_into_op_vals_value_opt_with_filter_int64,
};

use uuid::Uuid;

use crate::pb::fusion_flow::v1::{PageProcessRequest, PageProcessResponse, ProcessFilterRequest, SchedProcessDto};

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct ProcessDefinition {
  pub id: Uuid,
  pub tenant_id: i32,
  pub namespace_id: i64,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub variables: Option<serde_json::Value>,
  pub data: Option<Vec<u8>>,
  pub status: Option<ProcessStatus>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl PgRowType for ProcessDefinition {}

impl From<ProcessDefinition> for SchedProcessDto {
  fn from(row: ProcessDefinition) -> Self {
    SchedProcessDto {
      id: row.id.to_string(),
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

impl From<ProcessDefinition> for v1::ProcessDefinition {
  fn from(_value: ProcessDefinition) -> Self {
    todo!()
  }
}

#[derive(Debug, Fields)]
pub struct ProcessDefinitionForCreate {
  pub id: Uuid,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<serde_json::Value>,
}

impl From<CreateProcessRequest> for ProcessDefinitionForCreate {
  fn from(job: CreateProcessRequest) -> Self {
    Self {
      id: Uuid::now_v7(),
      description: job.description,
      tags: job.tags.map(|arr| arr.value).unwrap_or_default(),
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
  pub id: Option<OpValsUuid>,

  pub status: Option<OpValsInt32>,

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

impl TryFrom<ProcessFilterRequest> for ProcessDefinitionFilter {
  type Error = DataError;
  fn try_from(value: ProcessFilterRequest) -> Result<Self, Self::Error> {
    let f = Self {
      id: try_into_op_vals_uuid_with_filter_string(value.id)?,
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

impl From<PageResult<ProcessDefinition>> for PageProcessResponse {
  fn from(value: PageResult<ProcessDefinition>) -> Self {
    Self { page: Some(Page::new(value.page.total)), items: value.result.into_iter().map(Into::into).collect() }
  }
}
