use modelsql::{base::DbBmc, generate_pg_bmc_filter};

use super::{SchedNamespace, SchedNamespaceFilter, SchedNamespaceForUpdate};

pub struct SchedNamespaceBmc;
impl DbBmc for SchedNamespaceBmc {
  const TABLE: &'static str = "sched_namespace";
}

generate_pg_bmc_filter!(
  Bmc: SchedNamespaceBmc,
  Entity: SchedNamespace,
  Filter: SchedNamespaceFilter,
  ForUpdate: SchedNamespaceForUpdate,
);
