use modelsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use hetuiam_core::{
  infra::tables::TABLE_POLICY,
  types::{Policy, PolicyFilter, PolicyForCreate, PolicyForUpdate},
};

pub struct PolicyBmc;
impl DbBmc for PolicyBmc {
  const TABLE: &'static str = TABLE_POLICY;
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
