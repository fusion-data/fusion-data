use fusion_common::time::OffsetDateTime;
use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsString},
  postgres::PgRowType,
};
use o2o::o2o;
use sea_query::enum_def;
use sqlx::prelude::FromRow;

use crate::{
  pb::{CreatePermissionDto, UpdatePermissionDto},
  role::role_permission::RolePermissionFilter,
};

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct Permission {
  pub id: i64,
  pub code: String,
  pub description: String,
  pub resource: String,
  pub action: String,

  pub cid: i64,
  pub ctime: OffsetDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<OffsetDateTime>,
}
impl PgRowType for Permission {}

#[derive(Debug, Fields, o2o)]
#[from_owned(CreatePermissionDto)]
pub struct PermissionForCreate {
  pub code: String,
  pub description: Option<String>,
  pub resource: String,
  pub action: String,
}

#[derive(Debug, Fields, o2o)]
#[from_owned(UpdatePermissionDto)]
pub struct PermissionForUpdate {
  pub code: Option<String>,
  pub description: Option<String>,
  pub resource: Option<String>,
  pub action: Option<String>,
}

#[derive(Debug, Clone, Default, FilterNodes)]
pub struct PermissionFilter {
  pub id: Option<OpValsInt64>,
  pub code: Option<OpValsString>,
  pub description: Option<OpValsString>,
  pub resource: Option<OpValsString>,
  pub action: Option<OpValsString>,
}

#[derive(Debug, Clone, Default)]
pub struct PermissionFilters {
  pub filter: Vec<PermissionFilter>,
  pub role_perm_filter: RolePermissionFilter,
}
