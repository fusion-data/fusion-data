use fusion_common::time::OffsetDateTime;
use modelsql_core::filter::{OpValsInt32, OpValsString};
use serde::{Deserialize, Serialize};

use super::RolePermissionFilter;

/// Role status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i32)]
pub enum RoleStatus {
  Unspecified = 0,
  Disabled = 99,
  Enabled = 100,
}

impl From<i32> for RoleStatus {
  fn from(value: i32) -> Self {
    match value {
      99 => RoleStatus::Disabled,
      100 => RoleStatus::Enabled,
      _ => RoleStatus::Unspecified,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields, sqlx::FromRow), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct Role {
  pub id: i64,
  pub name: String,
  pub description: String,
  pub status: RoleStatus,
  pub cid: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub ctime: OffsetDateTime,
  pub mid: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub mtime: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct CreateRoleDto {
  pub name: String,
  pub description: Option<String>,
  pub status: Option<RoleStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RoleForUpdate {
  pub name: Option<String>,
  pub description: Option<String>,
  pub status: Option<RoleStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RoleFilter {
  pub name: Option<OpValsString>,
  pub description: Option<OpValsString>,
  pub status: Option<OpValsInt32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RoleFilters {
  pub filter: Vec<RoleFilter>,
  pub role_perm_filter: RolePermissionFilter,
}
