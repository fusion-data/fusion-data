use modelsql::{base::DbBmc, field::Fields, generate_pg_bmc_common, postgres::PgRowType};
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow, Fields)]
pub struct UserRole {
  pub user_id: i64,
  pub role_id: i64,
  pub ctime: i64,
  pub mtime: i64,
}
impl PgRowType for UserRole {}

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
  const TABLE: &'static str = "user_role";
}

generate_pg_bmc_common!(
  Bmc: UserRoleBmc,
  Entity: UserRole,
  ForCreate: UserRoleForCreate,
  ForUpdate: UserRoleForUpdate,
);
