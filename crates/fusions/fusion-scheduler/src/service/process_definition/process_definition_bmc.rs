use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns};

use super::{ProcessDefinition, ProcessDefinitionFilter, ProcessDefinitionForCreate, ProcessDefinitionForUpdate};

pub struct ProcessDefinitionBmc;
impl DbBmc for ProcessDefinitionBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "process_definition";
}

generate_common_bmc_fns!(
  Bmc: ProcessDefinitionBmc,
  Entity: ProcessDefinition,
  ForCreate: ProcessDefinitionForCreate,
  ForUpdate: ProcessDefinitionForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: ProcessDefinitionBmc,
  Entity: ProcessDefinition,
  Filter: ProcessDefinitionFilter,
);
