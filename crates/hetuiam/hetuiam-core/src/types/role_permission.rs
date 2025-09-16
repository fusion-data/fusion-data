use fusion_common::time::OffsetDateTime;
use modelsql_core::filter::OpValsInt64;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, modelsql::field::Fields), sea_query::enum_def)]
pub struct RolePermission {
  role_id: i64,
  permission_id: i64,
  cid: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  ctime: OffsetDateTime,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RolePermissionForCreate {
  pub role_id: i64,
  pub permission_id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RolePermissionFilter {
  pub role_id: Option<OpValsInt64>,
  pub permission_id: Option<OpValsInt64>,
}
