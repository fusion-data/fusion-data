use modelsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use super::{ProcessDefinition, ProcessDefinitionFilter, ProcessDefinitionForCreate, ProcessDefinitionForUpdate};

pub struct ProcessDefinitionBmc;
impl DbBmc for ProcessDefinitionBmc {
  const TABLE: &'static str = "process_definition";
}

generate_pg_bmc_common!(
  Bmc: ProcessDefinitionBmc,
  Entity: ProcessDefinition,
  ForCreate: ProcessDefinitionForCreate,
  ForUpdate: ProcessDefinitionForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: ProcessDefinitionBmc,
  Entity: ProcessDefinition,
  Filter: ProcessDefinitionFilter,
);
