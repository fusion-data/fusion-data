use chrono::{DateTime, FixedOffset};
use hetusql_core::filter::OpValInt64;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, hetusql::Fields), sea_query::enum_def)]
pub struct RolePermission {
  role_id: i64,
  permission_id: i64,
  created_by: i64,
  created_at: DateTime<FixedOffset>,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(hetusql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RolePermissionForCreate {
  pub role_id: i64,
  pub permission_id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(hetusql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RolePermissionFilter {
  pub role_id: Option<OpValInt64>,
  pub permission_id: Option<OpValInt64>,
}
