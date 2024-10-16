use ultimate_db::{base::DbBmc, generate_filter_bmc_fns};

use super::{SchedNamespace, SchedNamespaceFilter, SchedNamespaceForUpdate};

pub struct SchedNamespaceBmc;
impl DbBmc for SchedNamespaceBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_namespace";
}

generate_filter_bmc_fns!(
  Bmc: SchedNamespaceBmc,
  Entity: SchedNamespace,
  Filter: SchedNamespaceFilter,
  ForUpdate: SchedNamespaceForUpdate,
);
