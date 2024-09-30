use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns};

use super::{SchedTrigger, SchedTriggerFilter, SchedTriggerForCreate, SchedTriggerForUpdate};

pub struct TriggerBmc;
impl DbBmc for TriggerBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_trigger";
}

generate_common_bmc_fns!(
  Bmc: TriggerBmc,
  Entity: SchedTrigger,
  ForCreate: SchedTriggerForCreate,
  ForUpdate: SchedTriggerForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: TriggerBmc,
  Entity: SchedTrigger,
  Filter: SchedTriggerFilter,
);
