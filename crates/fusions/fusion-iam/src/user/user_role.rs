use modql::field::Fields;
use sqlx::prelude::FromRow;
use ultimate_db::{base::DbBmc, generate_common_bmc_fns, DbRowType};

#[derive(Debug, FromRow, Fields)]
pub struct UserRole {
  pub user_id: i64,
  pub role_id: i64,
  pub ctime: i64,
  pub mtime: i64,
}
impl DbRowType for UserRole {}

#[derive(Debug, Fields)]
pub struct UserRoleForCreate {
  pub user_id: i64,
  pub role_id: i64,
}

#[derive(Debug, Fields)]
pub struct UserRoleForUpdate {
  pub user_id: Option<i64>,
  pub role_id: Option<i64>,
}

pub struct UserRoleBmc;
impl DbBmc for UserRoleBmc {
  const SCHEMA: &'static str = "iam";
  const TABLE: &'static str = "user_role";
}

generate_common_bmc_fns!(
  Bmc: UserRoleBmc,
  Entity: UserRole,
  ForCreate: UserRoleForCreate,
  ForUpdate: UserRoleForUpdate,
);
