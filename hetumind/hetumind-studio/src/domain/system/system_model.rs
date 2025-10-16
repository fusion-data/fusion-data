use fusion_common::time::OffsetDateTime;
use fusionsql::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 设置表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "settings")]
pub struct Settings {
  pub key: String,
  pub value: String,
  pub load_on_startup: bool,
}

/// 变量表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "variables")]
pub struct Variables {
  pub id: String,
  pub key: String,
  pub kind: String,
  pub value: Option<String>,
}

/// 迁移表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "migrations")]
pub struct Migrations {
  pub id: i32,
  pub timestamp: OffsetDateTime,
  pub name: String,
}

/// 事件目标表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "event_destinations")]
pub struct EventDestinations {
  pub id: Uuid,
  pub destination: serde_json::Value,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_system_models() {
    assert_eq!(SettingsIden::Table.as_ref(), "settings");
    assert_eq!(VariablesIden::Table.as_ref(), "variables");
    assert_eq!(MigrationsIden::Table.as_ref(), "migrations");
    assert_eq!(EventDestinationsIden::Table.as_ref(), "event_destinations");
  }
}
