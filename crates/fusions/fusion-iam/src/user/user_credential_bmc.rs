use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns};

use super::{UserCredential, UserCredentialFilter, UserCredentialForCreate, UserCredentialForUpdate};

pub struct UserCredentialBmc;
impl DbBmc for UserCredentialBmc {
  const SCHEMA: &'static str = "iam";
  const TABLE: &'static str = "user_credential";
}

generate_common_bmc_fns!(
  Bmc: UserCredentialBmc,
  Entity: UserCredential,
  ForCreate: UserCredentialForCreate,
  ForUpdate: UserCredentialForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: UserCredentialBmc,
  Entity: UserCredential,
  Filter: UserCredentialFilter,
);
