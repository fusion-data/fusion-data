use ultimate_db::{base::DbBmc, generate_common_bmc_fns};

use super::{SchedJob, SchedJobForCreate, SchedJobForUpdate};

pub struct SchedJobBmc;
impl DbBmc for SchedJobBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_job";
}

generate_common_bmc_fns!(
  Bmc: SchedJobBmc,
  Entity: SchedJob,
  ForCreate: SchedJobForCreate,
  ForUpdate: SchedJobForUpdate,
);
