use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use chrono_tz::Tz;
use croner::Cron;
use duration_str::HumanFormat;
use fusion_scheduler_api::v1::{
  trigger_definition::{Schedule as ProtoSchedule, TriggerKind},
  CronSchedule, SimpleSchedule,
};
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64, OpValsString, OpValsValue},
};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgTypeInfo, FromRow};
use ulid::Ulid;
use ultimate::DataError;
use ultimate_api::v1::{PagePayload, Pagination};
use ultimate_common::time::UtcDateTime;
use ultimate_db::{datetime_to_sea_value, DbRowType};

use crate::pb::fusion_scheduler::v1::{PageTriggerRequest, PageTriggerResponse};

use super::util::cron_to_next_occurrence;

#[derive(Debug, Clone, FromRow, Fields)]
#[enum_def]
pub struct TriggerDefinition {
  pub id: i64,
  pub tenant_id: i32,
  pub namespace_id: i32,
  pub key: String,
  pub trigger_kind: i32,
  pub schedule: TriggerSchedule,
  pub variables: Option<serde_json::Value>,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub executed_count: i64,
  pub refresh_occurrence: UtcDateTime,
  pub status: i32,
  pub valid_time: Option<UtcDateTime>,
  pub invalid_time: Option<UtcDateTime>,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for TriggerDefinition {}

impl TriggerDefinition {
  pub fn is_valid_time(&self, now: &DateTime<Utc>) -> bool {
    let mut valid = true;
    if let Some(valid_time) = self.valid_time.as_ref() {
      valid = valid_time <= now;
    }
    if let Some(invalid_time) = self.invalid_time.as_ref() {
      valid = invalid_time >= now;
    }

    valid
  }
}

#[derive(Debug, Default, FilterNodes)]
pub struct TriggerDefinitionFilter {
  pub id: Option<OpValsInt64>,
  pub tenant_id: Option<OpValsInt32>,
  pub namespace_id: Option<OpValsInt32>,
  pub key: Option<OpValsString>,
  pub trigger_kind: Option<OpValsInt32>,
  pub status: Option<OpValsInt32>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub refresh_occurrence: Option<OpValsValue>,
  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub valid_time: Option<OpValsValue>,
  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub invalid_time: Option<OpValsValue>,

  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,
  #[modql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

#[derive(Debug, Fields)]
pub struct TriggerDefinitionForCreate {
  pub key: Option<String>,
  pub trigger_kind: Option<i32>,
  pub schedule: TriggerSchedule,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub data: Option<Vec<u8>>,

  // -- 以下字段应调用 Self::improve 自动判断补充。
  pub refresh_occurrence: Option<UtcDateTime>,
  pub valid_time: Option<UtcDateTime>,
  pub invalid_time: Option<UtcDateTime>,
}

impl TriggerDefinitionForCreate {
  pub fn improve(mut self) -> ultimate::Result<Self> {
    let begin = self.valid_time.unwrap_or(Utc::now());
    match &self.schedule {
      TriggerSchedule::Cron { cron, tz } => {
        self.refresh_occurrence = Some(cron_to_next_occurrence(cron, tz.as_deref(), &begin)?);
      }
      TriggerSchedule::Simple { interval, first_delay, .. } => {
        let refresh_occurrence = begin + first_delay.map(|fd| fd + *interval).unwrap_or(*interval);
        self.refresh_occurrence = Some(refresh_occurrence);
        if self.trigger_kind.is_none() || self.trigger_kind.is_some_and(|v| !(1..=2).contains(&v)) {
          self.trigger_kind = Some(TriggerKind::FixedRate as i32);
        }
      }
      _ => {}
    }
    if self.key.is_none() {
      self.key = Some(Ulid::new().to_string());
    }

    Ok(self)
  }
}

#[derive(Debug, Default, Fields)]
pub struct TriggerDefinitionForUpdate {
  pub trigger_kind: Option<i32>,
  pub schedule: Option<TriggerSchedule>,
  pub description: Option<String>,
  pub tags: Option<Vec<String>>,
  pub data: Option<Vec<u8>>,
  pub executed_count: Option<i64>,
  pub status: Option<i32>,
  pub refresh_occurrence: Option<UtcDateTime>,
  pub valid_time: Option<UtcDateTime>,
  pub invalid_time: Option<UtcDateTime>,
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
  Simple { interval: Duration, first_delay: Option<Duration>, execution_count: Option<i64> },
  Cron { cron: String, tz: Option<String> },
  Depend,
}

impl From<TriggerSchedule> for sea_query::Value {
  fn from(value: TriggerSchedule) -> Self {
    sea_query::Value::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
  }
}

impl sea_query::Nullable for TriggerSchedule {
  fn null() -> sea_query::Value {
    sea_query::Value::Json(None)
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
      TriggerSchedule::Cron { cron, tz } => ProtoSchedule::Cron(CronSchedule { cron, tz }),
      TriggerSchedule::Depend => ProtoSchedule::Depend(true),
    }
  }
}

impl TryFrom<CronSchedule> for TriggerSchedule {
  type Error = DataError;

  fn try_from(value: CronSchedule) -> Result<Self, Self::Error> {
    let mut cron = Cron::new(&value.cron);
    cron.parse().map_err(|e| DataError::bad_request(e.to_string()))?;
    Ok(TriggerSchedule::Cron { cron: value.cron, tz: value.tz })
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
    let cron = TriggerSchedule::Cron { cron: "0 0 1 * * * ?".to_string(), tz: Some("Asia/Chongqing".to_string()) };

    println!("{}", serde_json::to_string_pretty(&depend).unwrap());
    println!("{}", serde_json::to_string_pretty(&simple).unwrap());
    println!("{}", serde_json::to_string_pretty(&cron).unwrap());
  }

  #[test]
  fn test_trigger_for_create() {
    let c = TriggerDefinitionForCreate {
      key: None,
      trigger_kind: None,
      schedule: TriggerSchedule::Simple { interval: Duration::from_secs(30), first_delay: None, execution_count: None },
      description: None,
      tags: vec![],
      data: None,
      refresh_occurrence: None,
      valid_time: None,
      invalid_time: None,
    }
    .improve()
    .unwrap();
    println!("c: {:?}", c);
  }
}
