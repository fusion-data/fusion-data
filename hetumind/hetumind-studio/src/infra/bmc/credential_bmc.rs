use fusionsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use crate::domain::credential::{CredentialEntity, CredentialFilter, CredentialForInsert, CredentialForUpdate};

pub struct CredentialBmc;
impl DbBmc for CredentialBmc {
  const TABLE: &str = "credential_entity";

  fn _use_logical_deletion() -> bool {
    true
  }
}

generate_pg_bmc_common!(
  Bmc: CredentialBmc,
  Entity: CredentialEntity,
  ForUpdate: CredentialForUpdate,
  ForInsert: CredentialForInsert,
);

generate_pg_bmc_filter!(
  Bmc: CredentialBmc,
  Entity: CredentialEntity,
  Filter: CredentialFilter,
  ForUpdate: CredentialForUpdate,
);
