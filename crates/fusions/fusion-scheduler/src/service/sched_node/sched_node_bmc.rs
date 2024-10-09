use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns};

use super::{SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate};

pub struct SchedNodeBmc;
impl DbBmc for SchedNodeBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_node";
}

generate_common_bmc_fns!(
  Bmc: SchedNodeBmc,
  Entity: SchedNode,
  ForCreate: SchedNodeForCreate,
  ForUpdate: SchedNodeForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: SchedNodeBmc,
  Entity: SchedNode,
  Filter: SchedNodeFilter,
);
