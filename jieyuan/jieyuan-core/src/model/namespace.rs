use fusion_common::time::{DateTime, FixedOffset};
use fusionsql_core::filter::{OpValDateTime, OpValInt32, OpValInt64, OpValString};
use fusionsql_core::page::Page;
use serde::{Deserialize, Serialize};

/// Namespace status enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i32)]
pub enum NamespaceStatus {
  #[default]
  Disabled = 99,
  Active = 100,
}

impl From<i32> for NamespaceStatus {
  fn from(value: i32) -> Self {
    match value {
      100 => NamespaceStatus::Active,
      99 => NamespaceStatus::Disabled,
      _ => NamespaceStatus::Disabled,
    }
  }
}

// Sea Query trait implementations for database compatibility
#[cfg(feature = "with-db")]
impl From<NamespaceStatus> for sea_query::Value {
  fn from(status: NamespaceStatus) -> Self {
    sea_query::Value::Int(Some(status as i32))
  }
}

#[cfg(feature = "with-db")]
impl sea_query::Nullable for NamespaceStatus {
  fn null() -> sea_query::Value {
    sea_query::Value::Int(None)
  }
}

/// Namespace entity - core database table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(sqlx::FromRow, fusionsql::Fields),
  sea_query::enum_def(table_name = "iam_namespace")
)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct NamespaceEntity {
  pub id: i64,
  pub tenant_id: i64,
  pub name: String,
  pub description: Option<String>,
  pub status: NamespaceStatus,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}

/// Namespace creation request model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct NamespaceForCreate {
  pub name: String,
  pub description: Option<String>,
  pub status: Option<NamespaceStatus>,
}

/// Namespace update request model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct NamespaceForUpdate {
  pub name: Option<String>,
  pub description: Option<String>,
  pub status: Option<NamespaceStatus>,
}

/// Namespace pagination query model
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct NamespaceForPage {
  #[serde(default)]
  pub page: Page,
  #[serde(default)]
  pub filters: Vec<NamespaceFilter>,
}

/// Namespace filter for query operations
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct NamespaceFilter {
  pub id: Option<OpValInt64>,
  pub tenant_id: Option<OpValInt64>,
  pub name: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub description: Option<OpValString>,
  pub created_by: Option<OpValInt64>,
  pub created_at: Option<OpValDateTime>,
  pub updated_by: Option<OpValInt64>,
  pub updated_at: Option<OpValDateTime>,
}

/// Table name constant
pub const TABLE_NAMESPACE: &str = "iam_namespace";
