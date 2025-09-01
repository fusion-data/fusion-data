use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use fusion_common::time::OffsetDateTime;

/// 洞察元数据表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "insights_metadata")]
pub struct InsightsMetadata {
  pub meta_id: i32,
  pub workflow_id: Option<String>,
  pub project_id: Option<String>,
  pub workflow_name: String,
  pub project_name: String,
}

/// 按周期洞察表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "insights_by_period")]
pub struct InsightsByPeriod {
  pub id: i32,
  pub meta_id: i32,
  pub kind: i32,
  pub value: i32,
  pub period_unit: i32,
  pub period_start: Option<OffsetDateTime>,
}

/// 原始洞察表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "insights_raw")]
pub struct InsightsRaw {
  pub id: i32,
  pub meta_id: i32,
  pub kind: i32,
  pub value: i32,
  pub timestamp: OffsetDateTime,
}

/// 洞察类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightKind {
  /// 节省时间（分钟）
  TimeSavedMinutes = 0,
  /// 运行时间（毫秒）
  RuntimeMilliseconds = 1,
  /// 成功
  Success = 2,
  /// 失败
  Failure = 3,
}

/// 周期单位枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeriodUnit {
  /// 小时
  Hour = 0,
  /// 天
  Day = 1,
  /// 周
  Week = 2,
}

impl From<i32> for InsightKind {
  fn from(value: i32) -> Self {
    match value {
      0 => InsightKind::TimeSavedMinutes,
      1 => InsightKind::RuntimeMilliseconds,
      2 => InsightKind::Success,
      3 => InsightKind::Failure,
      _ => InsightKind::TimeSavedMinutes, // 默认值
    }
  }
}

impl From<InsightKind> for i32 {
  fn from(value: InsightKind) -> Self {
    value as i32
  }
}

impl From<i32> for PeriodUnit {
  fn from(value: i32) -> Self {
    match value {
      0 => PeriodUnit::Hour,
      1 => PeriodUnit::Day,
      2 => PeriodUnit::Week,
      _ => PeriodUnit::Hour, // 默认值
    }
  }
}

impl From<PeriodUnit> for i32 {
  fn from(value: PeriodUnit) -> Self {
    value as i32
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_insights_models() {
    assert_eq!(InsightsMetadataIden::Table.as_ref(), "insights_metadata");
    assert_eq!(InsightsByPeriodIden::Table.as_ref(), "insights_by_period");
    assert_eq!(InsightsRawIden::Table.as_ref(), "insights_raw");
  }

  #[test]
  fn test_insight_kind_conversion() {
    assert_eq!(InsightKind::from(0), InsightKind::TimeSavedMinutes);
    assert_eq!(InsightKind::from(1), InsightKind::RuntimeMilliseconds);
    assert_eq!(InsightKind::from(2), InsightKind::Success);
    assert_eq!(InsightKind::from(3), InsightKind::Failure);

    assert_eq!(i32::from(InsightKind::TimeSavedMinutes), 0);
    assert_eq!(i32::from(InsightKind::RuntimeMilliseconds), 1);
    assert_eq!(i32::from(InsightKind::Success), 2);
    assert_eq!(i32::from(InsightKind::Failure), 3);
  }

  #[test]
  fn test_period_unit_conversion() {
    assert_eq!(PeriodUnit::from(0), PeriodUnit::Hour);
    assert_eq!(PeriodUnit::from(1), PeriodUnit::Day);
    assert_eq!(PeriodUnit::from(2), PeriodUnit::Week);

    assert_eq!(i32::from(PeriodUnit::Hour), 0);
    assert_eq!(i32::from(PeriodUnit::Day), 1);
    assert_eq!(i32::from(PeriodUnit::Week), 2);
  }
}
