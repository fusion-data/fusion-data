use modelsql::{base::DbBmc, generate_common_bmc_fns};

use super::*;

pub struct ProcessInstanceBmc;
impl DbBmc for ProcessInstanceBmc {
  const TABLE: &'static str = "process_instance";
}

generate_common_bmc_fns!(
  Bmc: ProcessInstanceBmc,
  Entity: ProcessInstance,
  ForCreate: ProcessInstanceForCreate,
  ForUpdate: ProcessInstanceForUpdate,
);
