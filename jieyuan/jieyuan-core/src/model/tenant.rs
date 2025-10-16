use fusion_common::time::{DateTime, FixedOffset};
use fusionsql::page::Page;
use fusionsql_core::filter::{OpValDateTime, OpValInt32, OpValInt64, OpValString};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Tenant status enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i32)]
pub enum TenantStatus {
  #[default]
  Inactive = 99,
  Active = 100,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct Tenant {
  pub id: i64,
  pub name: String,
  pub description: Option<String>,
  pub status: TenantStatus,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
  pub logical_deletion: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantForCreate {
  pub name: String,
  pub description: Option<String>,
  pub status: Option<TenantStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantForUpdate {
  pub name: Option<String>,
  pub description: Option<String>,
  pub status: Option<TenantStatus>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantForPage {
  pub page: Page,
  pub filters: Vec<TenantFilter>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantFilter {
  pub id: Option<OpValInt64>,
  pub name: Option<OpValString>,
  pub description: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub created_by: Option<OpValInt64>,
  pub created_at: Option<OpValDateTime>,
  pub updated_by: Option<OpValInt64>,
  pub updated_at: Option<OpValDateTime>,
}
