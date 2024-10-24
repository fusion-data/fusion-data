use sea_query::enum_def;
use sqlx::prelude::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsString},
};
use ultimate_db::DbRowType;

use crate::pb::fusion_iam::v1::RoleStatus;

use super::role_permission::RolePermissionFilter;

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct Role {
  pub id: i64,
  pub name: String,
  pub description: String,
  pub status: RoleStatus,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for Role {}

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
