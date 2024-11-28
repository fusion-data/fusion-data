use ultimate_db::{base::DbBmc, generate_common_bmc_fns};

use super::{Policy, PolicyForCreate, PolicyForUpdate};

pub struct PolicyBmc;
impl DbBmc for PolicyBmc {

  const TABLE: &'static str = "policy";
}

generate_common_bmc_fns!(
  Bmc: PolicyBmc,
  Entity: Policy,
  ForCreate: PolicyForCreate,
  ForUpdate: PolicyForUpdate,
);
