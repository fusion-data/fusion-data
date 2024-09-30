use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsValue},
};
use sqlx::FromRow;
use ultimate::DataError;
use ultimate_api::v1::{Page, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::{datetime_to_sea_value, uuid_to_sea_value, DbRowType};
use uuid::Uuid;

use crate::pb::fusion_scheduler::v1::{PageJobTaskRequest, PageJobTaskResponse};

#[derive(Debug, FromRow, Fields)]
pub struct SchedJobTask {
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
impl DbRowType for SchedJobTask {}

#[derive(Debug, Default, FilterNodes)]
pub struct JobTaskFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub id: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub job_id: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub trigger_id: Option<OpValsValue>,

  pub status: Option<OpValsInt32>,

  pub retry_count: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub execute_begin_time: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub execute_end_time: Option<OpValsValue>,
}

pub struct JobTaskForPage {
  pub pagination: Pagination,
  pub filter: Vec<JobTaskFilter>,
}

impl TryFrom<PageJobTaskRequest> for JobTaskForPage {
  type Error = DataError;

  fn try_from(value: PageJobTaskRequest) -> Result<Self, Self::Error> {
    todo!()
  }
}

pub struct JobTaskPage {
  pub page: Page,
  pub items: Vec<SchedJobTask>,
}

impl From<JobTaskPage> for PageJobTaskResponse {
  fn from(value: JobTaskPage) -> Self {
    todo!()
  }
}
