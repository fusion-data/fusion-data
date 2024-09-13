use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64},
};
use sea_query::enum_def;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use ultimate_db::{
  base::{self, DbBmc},
  DbRowType, ModelManager, Result,
};

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct RolePermission {
  role_id: i64,
  permission_id: i64,
  cid: i64,
  ctime: UtcDateTime,
}
impl DbRowType for RolePermission {}

#[derive(Debug, Fields)]
pub struct RolePermissionForCreate {
  pub role_id: i64,
  pub permission_id: i64,
}

#[derive(Debug, Clone, Default, FilterNodes)]
pub struct RolePermissionFilter {
  pub role_id: Option<OpValsInt64>,
  pub permission_id: Option<OpValsInt64>,
}

pub struct RolePermissionBmc;
impl DbBmc for RolePermissionBmc {
  const SCHEMA: &'static str = "iam";
  const TABLE: &'static str = "role_permission";

  fn has_modification_timestamps() -> bool {
    false
  }
}

impl RolePermissionBmc {
  pub async fn insert_many(mm: &ModelManager, data: Vec<RolePermissionForCreate>) -> Result<u64> {
    base::insert_many::<Self, _>(mm, data).await
  }
}
