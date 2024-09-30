use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsValue},
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate::DataError;
use ultimate_api::v1::{Page, PagePayload, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::{datetime_to_sea_value, uuid_to_sea_value, DbRowType};
use uuid::Uuid;

use crate::pb::fusion_scheduler::v1::{PageTriggerRequest, PageTriggerResponse};

#[derive(Debug, Clone, FromRow, Fields)]
#[enum_def]
pub struct SchedTrigger {
  pub id: Uuid,
  pub r#type: i32,
  pub schedule: serde_json::Value,
  pub description: Option<String>,
  pub tags: Vec<String>,
  // json object
  pub data: Option<Vec<u8>>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for SchedTrigger {}

#[derive(Debug, FilterNodes)]
pub struct SchedTriggerFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub id: Option<OpValsValue>,

  pub r#type: Option<OpValsInt32>,

  pub status: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

#[derive(Debug, Fields)]
pub struct SchedTriggerForCreate {
  pub r#type: i32,
  pub schedule: serde_json::Value,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub data: Option<Vec<u8>>,
}

#[derive(Debug, Fields)]
pub struct SchedTriggerForUpdate {
  pub r#type: Option<i32>,
  pub schedule: Option<serde_json::Value>,
  pub description: Option<String>,
  pub tags: Option<Vec<String>>,
  pub data: Option<Vec<u8>>,
}

pub struct SchedTriggerForPage {
  pub pagination: Pagination,
  pub filter: Vec<SchedTriggerFilter>,
}

impl TryFrom<PageTriggerRequest> for SchedTriggerForPage {
  type Error = DataError;

  fn try_from(value: PageTriggerRequest) -> Result<Self, Self::Error> {
    todo!()
  }
}

impl From<PagePayload<SchedTrigger>> for PageTriggerResponse {
  fn from(value: PagePayload<SchedTrigger>) -> Self {
    todo!()
  }
}
