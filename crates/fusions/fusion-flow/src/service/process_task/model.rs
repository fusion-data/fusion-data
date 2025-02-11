use sqlx::FromRow;
use ultimate::DataError;
use ultimate_api::v1::{Page, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64, OpValsValue},
};
use ultimate_db::{datetime_to_sea_value, uuid_to_sea_value, DbRowType};
use uuid::Uuid;

use crate::pb::fusion_flow::v1::{PageProcessTaskRequest, PageProcessTaskResponse};

#[derive(Debug, FromRow, Fields)]
pub struct ProcessTask {
  pub id: Uuid,
  pub job_id: Uuid,
  pub trigger_id: Option<Uuid>,
  pub status: i32,
  pub retry_count: i32,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for ProcessTask {}

#[derive(Debug, Default, FilterNodes)]
pub struct ProcessTaskFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub id: Option<OpValsValue>,

  pub process_id: Option<OpValsInt64>,

  pub trigger_id: Option<OpValsInt64>,

  pub status: Option<OpValsInt32>,

  pub retry_count: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub execute_begin_time: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub execute_end_time: Option<OpValsValue>,
}

pub struct JobTaskForPage {
  pub pagination: Pagination,
  pub filter: Vec<ProcessTaskFilter>,
}

impl TryFrom<PageProcessTaskRequest> for JobTaskForPage {
  type Error = DataError;

  fn try_from(value: PageProcessTaskRequest) -> Result<Self, Self::Error> {
    todo!()
  }
}

pub struct JobTaskPage {
  pub page: Page,
  pub items: Vec<ProcessTask>,
}

impl From<JobTaskPage> for PageProcessTaskResponse {
  fn from(value: JobTaskPage) -> Self {
    todo!()
  }
}
