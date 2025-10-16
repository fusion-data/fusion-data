use chrono::{DateTime, FixedOffset};
use fusion_common::page::Page;
use fusionsql_core::filter::{OpValInt32, OpValInt64, OpValString};
use serde::{Deserialize, Serialize};

/// Policy decision effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum DecisionEffect {
  Allow,
  Deny,
}

/// Policy statement representing a single permission rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyStatement {
  /// Statement ID (optional)
  pub sid: Option<String>,
  /// Effect of the statement (Allow or Deny)
  pub effect: DecisionEffect,
  /// Actions that are permitted or denied
  pub action: Vec<String>,
  /// Resources that the actions apply to
  pub resource: Vec<String>,
  /// Conditions for evaluation (optional)
  pub condition: Option<serde_json::Value>,
}

/// Policy document containing version and statements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyDocument {
  /// Policy version
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, examples("2025-01-01")))]
  pub version: String,
  /// Policy ID (optional)
  pub id: Option<String>,
  /// List of policy statements
  pub statement: Vec<PolicyStatement>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyEntity {
  pub id: i64,
  pub tenant_id: i64,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: i32,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
  pub logical_deletion: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyForCreate {
  pub tenant_id: i64,
  pub description: Option<String>,
  pub policy: serde_json::Value,
  pub status: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyForUpdate {
  pub description: Option<String>,
  pub policy: Option<serde_json::Value>,
  pub status: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyForPage {
  pub page: Page,
  pub filter: Vec<PolicyFilter>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PolicyFilter {
  pub id: Option<OpValInt64>,
  pub tenant_id: Option<OpValInt64>,
  pub description: Option<OpValString>,
  pub status: Option<OpValInt32>,
}
