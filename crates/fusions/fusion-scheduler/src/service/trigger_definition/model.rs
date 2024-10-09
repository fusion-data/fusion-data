use std::time::Duration;

use croner::Cron;
use duration_str::HumanFormat;
use fusion_scheduler_api::v1::{trigger_definition::Schedule as ProtoSchedule, CronSchedule, SimpleSchedule};
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsValue},
};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, FromRow};
use ultimate::DataError;
use ultimate_api::v1::{PagePayload, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::{datetime_to_sea_value, uuid_to_sea_value, DbRowType};
use uuid::Uuid;

use crate::pb::fusion_scheduler::v1::{PageTriggerRequest, PageTriggerResponse};

#[derive(Debug, Clone, FromRow, Fields)]
#[enum_def]
pub struct TriggerDefinition {
  pub id: Uuid,
  pub kind: i32,
  pub schedule: TriggerSchedule,
  pub variables: Option<serde_json::Value>,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for TriggerDefinition {}

#[derive(Debug, FilterNodes)]
pub struct TriggerDefinitionFilter {
  #[modql(to_sea_value_fn = "uuid_to_sea_value")]
  pub id: Option<OpValsValue>,

  pub kind: Option<OpValsInt32>,

  pub status: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

#[derive(Debug, Fields)]
pub struct TriggerDefinitionForCreate {
  pub kind: i32,
  pub schedule: serde_json::Value,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub data: Option<Vec<u8>>,
}

#[derive(Debug, Fields)]
pub struct TriggerDefinitionForUpdate {
  pub kind: Option<i32>,
  pub schedule: Option<serde_json::Value>,
  pub description: Option<String>,
  pub tags: Option<Vec<String>>,
  pub data: Option<Vec<u8>>,
}

pub struct TriggerDefinitionForPage {
  pub pagination: Pagination,
  pub filter: Vec<TriggerDefinitionFilter>,
}

impl TryFrom<PageTriggerRequest> for TriggerDefinitionForPage {
  type Error = DataError;

  fn try_from(value: PageTriggerRequest) -> Result<Self, Self::Error> {
    todo!()
  }
}

impl From<PagePayload<TriggerDefinition>> for PageTriggerResponse {
  fn from(value: PagePayload<TriggerDefinition>) -> Self {
    todo!()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TriggerSchedule {
  Simple { interval: Duration, first_delay: Option<Duration>, execution_count: Option<i32> },
  Cron { cron: String },
  Depend,
}

impl From<TriggerSchedule> for sea_query::Value {
  fn from(value: TriggerSchedule) -> Self {
    sea_query::Value::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}

impl sqlx::Type<sqlx::Postgres> for TriggerSchedule {
  fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
    PgTypeInfo::with_name("jsonb")
  }
}

impl sqlx::Decode<'_, sqlx::Postgres> for TriggerSchedule {
  fn decode(
    value: <sqlx::Postgres as sqlx::database::HasValueRef<'_>>::ValueRef,
  ) -> Result<Self, sqlx::error::BoxDynError> {
    let v = serde_json::from_slice(value.as_bytes()?)?;
    Ok(v)
  }
}

impl TryFrom<SimpleSchedule> for TriggerSchedule {
  type Error = DataError;

  fn try_from(value: SimpleSchedule) -> Result<Self, Self::Error> {
    let interval = duration_str::parse_std(value.interval).map_err(DataError::bad_request)?;
    let first_delay = Some(duration_str::parse_std(value.first_delay).map_err(DataError::bad_request)?);
    let execution_count = value.execution_count;
    Ok(TriggerSchedule::Simple { interval, first_delay, execution_count })
  }
}

impl From<TriggerSchedule> for ProtoSchedule {
  fn from(value: TriggerSchedule) -> Self {
    match value {
      TriggerSchedule::Simple { interval, first_delay, execution_count } => ProtoSchedule::Simple(SimpleSchedule {
        interval: interval.human_format(),
        first_delay: first_delay.map(|d| d.human_format()).unwrap_or_else(|| "0s".to_string()),
        execution_count,
      }),
      TriggerSchedule::Cron { cron } => ProtoSchedule::Cron(CronSchedule { cron }),
      TriggerSchedule::Depend => ProtoSchedule::Depend(true),
    }
  }
}

impl TryFrom<CronSchedule> for TriggerSchedule {
  type Error = DataError;

  fn try_from(value: CronSchedule) -> Result<Self, Self::Error> {
    let mut cron = Cron::new(&value.cron);
    cron.parse().map_err(|e| DataError::bad_request(e.to_string()))?;
    Ok(TriggerSchedule::Cron { cron: value.cron })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_trigger_schedule() {
    let depend = TriggerSchedule::Depend;
    let simple = TriggerSchedule::Simple {
      interval: Duration::from_secs(10),
      first_delay: Some(Duration::from_secs(30)),
      execution_count: Some(5),
    };
    let cron = TriggerSchedule::Cron { cron: "0 0 1 * * * ?".to_string() };

    println!("{}", serde_json::to_string_pretty(&depend).unwrap());
    println!("{}", serde_json::to_string_pretty(&simple).unwrap());
    println!("{}", serde_json::to_string_pretty(&cron).unwrap());
  }
}
