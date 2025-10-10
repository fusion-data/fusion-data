use fusionsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use jieyuan_core::model::{Policy, PolicyFilter, PolicyForCreate, PolicyForUpdate, TABLE_POLICY};

pub struct PolicyBmc;
impl DbBmc for PolicyBmc {
  const TABLE: &'static str = TABLE_POLICY;
  fn _use_logical_deletion() -> bool {
    true
  }
}

generate_pg_bmc_common!(
  Bmc: PolicyBmc,
  Entity: Policy,
  ForCreate: PolicyForCreate,
  ForUpdate: PolicyForUpdate,
  ForInsert: PolicyForCreate,
);
generate_pg_bmc_filter!(
  Bmc: PolicyBmc,
  Entity: Policy,
  Filter: PolicyFilter,
);
