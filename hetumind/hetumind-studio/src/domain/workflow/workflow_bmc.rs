use std::sync::OnceLock;

use fusionsql::{
  base::{BmcConfig, DbBmc},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use hetumind_core::workflow::{WorkflowFilter, WorkflowForCreate, WorkflowForUpdate};

use super::WorkflowEntity;

pub struct WorkflowBmc;
impl DbBmc for WorkflowBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("workflow_entity"))
  }
}
generate_pg_bmc_common!(
  Bmc: WorkflowBmc,
  Entity: WorkflowEntity,
  ForUpdate: WorkflowForUpdate,
  ForInsert: WorkflowForCreate,
);
generate_pg_bmc_filter!(
  Bmc: WorkflowBmc,
  Entity: WorkflowEntity,
  Filter: WorkflowFilter,
  ForUpdate: WorkflowForUpdate,
);
