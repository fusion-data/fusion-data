use fusion_common::time::OffsetDateTime;
use fusionsql::filter::Page;
use fusionsql_core::filter::{OpValInt64, OpValString};
use serde::{Deserialize, Serialize};

use super::RolePermissionFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(
  feature = "with-db",
  derive(sqlx::FromRow, fusionsql::field::Fields),
  sea_query::enum_def(table_name = "iam_permission")
)]
pub struct Permission {
  pub id: i64,
  pub code: String,
  pub description: String,
  pub resource: String,
  pub action: String,
  pub created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PermissionForCreate {
  pub code: String,
  pub description: Option<String>,
  pub resource: String,
  pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PermissionForUpdate {
  pub code: Option<String>,
  pub description: Option<String>,
  pub resource: Option<String>,
  pub action: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PermissionFilter {
  pub id: Option<OpValInt64>,
  pub code: Option<OpValString>,
  pub description: Option<OpValString>,
  pub resource: Option<OpValString>,
  pub action: Option<OpValString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct PermissionForPage {
  pub page: Page,
  pub filters: Vec<PermissionFilter>,
  pub role_perm_filter: RolePermissionFilter,
}
