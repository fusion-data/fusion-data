use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns};

use super::{TriggerDefinition, TriggerDefinitionFilter, TriggerDefinitionForCreate, TriggerDefinitionForUpdate};

pub struct TriggerDefinitionBmc;
impl DbBmc for TriggerDefinitionBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "trigger_definition";
}

generate_common_bmc_fns!(
  Bmc: TriggerDefinitionBmc,
  Entity: TriggerDefinition,
  ForCreate: TriggerDefinitionForCreate,
  ForUpdate: TriggerDefinitionForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: TriggerDefinitionBmc,
  Entity: TriggerDefinition,
  Filter: TriggerDefinitionFilter,
);
