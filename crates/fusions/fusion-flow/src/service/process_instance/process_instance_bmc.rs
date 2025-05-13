use modelsql::{base::DbBmc, generate_pg_bmc_common};

use super::*;

#[allow(unused)]
pub struct ProcessInstanceBmc;
impl DbBmc for ProcessInstanceBmc {
  const TABLE: &'static str = "process_instance";
}

generate_pg_bmc_common!(
  Bmc: ProcessInstanceBmc,
  Entity: ProcessInstance,
  ForCreate: ProcessInstanceForCreate,
  ForUpdate: ProcessInstanceForUpdate,
);
