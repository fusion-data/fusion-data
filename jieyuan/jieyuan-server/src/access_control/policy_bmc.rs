use std::sync::OnceLock;

use fusionsql::{
  base::{BmcConfig, DbBmc},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use jieyuan_core::model::{PolicyEntity, PolicyFilter, PolicyForCreate, PolicyForUpdate, TABLE_POLICY};

pub struct PolicyBmc;
impl DbBmc for PolicyBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_POLICY).with_use_logical_deletion(true))
  }
}

generate_pg_bmc_common!(
  Bmc: PolicyBmc,
  Entity: PolicyEntity,
  ForCreate: PolicyForCreate,
  ForUpdate: PolicyForUpdate,
  ForInsert: PolicyForCreate,
);
generate_pg_bmc_filter!(
  Bmc: PolicyBmc,
  Entity: PolicyEntity,
  Filter: PolicyFilter,
);
