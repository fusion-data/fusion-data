use fusionsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};
use hetumind_core::workflow::{WorkflowFilter, WorkflowForCreate, WorkflowForUpdate};

use super::WorkflowEntity;

pub struct WorkflowBmc;
impl DbBmc for WorkflowBmc {
  const TABLE: &'static str = "workflow_entity";
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
