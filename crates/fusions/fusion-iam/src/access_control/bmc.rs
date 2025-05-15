use modelsql::{base::DbBmc, generate_pg_bmc_common};

use super::{Policy, PolicyForCreate, PolicyForUpdate};

pub struct PolicyBmc;
impl DbBmc for PolicyBmc {
  const TABLE: &'static str = "policy";
}

generate_pg_bmc_common!(
  Bmc: PolicyBmc,
  Entity: Policy,
  ForCreate: PolicyForCreate,
  ForUpdate: PolicyForUpdate,
  ForInsert: PolicyForCreate,
);
