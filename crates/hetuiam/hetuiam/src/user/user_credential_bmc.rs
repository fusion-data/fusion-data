use modelsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use super::{UserCredential, UserCredentialFilter, UserCredentialForCreate, UserCredentialForUpdate};

pub struct UserCredentialBmc;
impl DbBmc for UserCredentialBmc {
  const TABLE: &'static str = "user_credential";
}

generate_pg_bmc_common!(
  Bmc: UserCredentialBmc,
  Entity: UserCredential,
  ForCreate: UserCredentialForCreate,
  ForUpdate: UserCredentialForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: UserCredentialBmc,
  Entity: UserCredential,
  Filter: UserCredentialFilter,
);
