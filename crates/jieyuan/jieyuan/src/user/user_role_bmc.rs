use modelsql::{base::DbBmc, generate_pg_bmc_common};

use jieyuan_core::model::{TABLE_USER_ROLE, UserRole, UserRoleForCreate, UserRoleForUpdate};

pub struct UserRoleBmc;
impl DbBmc for UserRoleBmc {
  const TABLE: &'static str = TABLE_USER_ROLE;
}

generate_pg_bmc_common!(
  Bmc: UserRoleBmc,
  Entity: UserRole,
  ForCreate: UserRoleForCreate,
  ForUpdate: UserRoleForUpdate,
  ForInsert: UserRoleForCreate,
);
