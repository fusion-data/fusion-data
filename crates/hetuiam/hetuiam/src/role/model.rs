use fusion_common::time::OffsetDateTime;
use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsString},
  postgres::PgRowType,
};
use sea_query::enum_def;
use sqlx::prelude::FromRow;

use crate::pb::RoleStatus;

use super::role_permission::RolePermissionFilter;

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct Role {
  pub id: i64,
  pub name: String,
  pub description: String,
  pub status: RoleStatus,
  pub cid: i64,
  pub ctime: OffsetDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<OffsetDateTime>,
}
impl PgRowType for Role {}

impl From<RoleStatus> for sea_query::Value {
  fn from(value: RoleStatus) -> Self {
    sea_query::Value::Int(Some(value as i32))
  }
}
impl sea_query::Nullable for RoleStatus {
  fn null() -> sea_query::Value {
    sea_query::Value::Int(None)
  }
}

#[derive(Debug, Fields)]
pub struct RoleForUpdate {
  pub name: Option<String>,
  pub description: Option<String>,
  pub status: Option<RoleStatus>,
}

#[derive(Debug, Clone, FilterNodes)]
pub struct RoleFilter {
  pub name: Option<OpValsString>,
  pub description: Option<OpValsString>,
  pub status: Option<OpValsInt32>,
}

#[derive(Debug, Clone, Default)]
pub struct RoleFilters {
  pub filter: Vec<RoleFilter>,
  pub role_perm_filter: RolePermissionFilter,
}
