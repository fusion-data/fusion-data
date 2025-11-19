use std::sync::OnceLock;

use fusionsql::{
  base::{BmcConfig, DbBmc},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use hetumind_core::workflow::{ExecutionFilter, ExecutionForUpdate};

use crate::domain::workflow::ExecutionDataEntity;

use super::ExecutionEntity;

pub struct ExecutionBmc;
impl DbBmc for ExecutionBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("execution_entity"))
  }
}
generate_pg_bmc_common!(
  Bmc: ExecutionBmc,
  Entity: ExecutionEntity,
  ForUpdate: ExecutionForUpdate,
  ForInsert: ExecutionEntity,
);
generate_pg_bmc_filter!(
  Bmc: ExecutionBmc,
  Entity: ExecutionEntity,
  Filter: ExecutionFilter,
  ForUpdate: ExecutionForUpdate,
);

pub struct ExecutionDataBmc;
impl DbBmc for ExecutionDataBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("execution_data"))
  }
}
generate_pg_bmc_common!(
  Bmc: ExecutionDataBmc,
  Entity: ExecutionDataEntity,
  ForInsert: ExecutionDataEntity,
);
