use fusion_common::time::OffsetDateTime;
use fusionsql_core::filter::OpValInt64;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields), sea_query::enum_def)]
pub struct RolePermission {
  role_id: i64,
  permission_id: i64,
  created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  created_at: OffsetDateTime,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RolePermissionForCreate {
  pub role_id: i64,
  pub permission_id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RolePermissionFilter {
  pub role_id: Option<OpValInt64>,
  pub permission_id: Option<OpValInt64>,
}
