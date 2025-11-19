use std::sync::OnceLock;

use fusionsql::{
  base::{BmcConfig, DbBmc},
  generate_pg_bmc_common,
};

use jieyuan_core::model::{TABLE_USER_ROLE, UserRole, UserRoleForCreate, UserRoleForUpdate};

pub struct UserRoleBmc;
impl DbBmc for UserRoleBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_USER_ROLE))
  }
}

generate_pg_bmc_common!(
  Bmc: UserRoleBmc,
  Entity: UserRole,
  ForCreate: UserRoleForCreate,
  ForUpdate: UserRoleForUpdate,
  ForInsert: UserRoleForCreate,
);
