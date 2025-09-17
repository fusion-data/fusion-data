use modelsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use jieyuan_core::model::{
  TABLE_USER_CREDENTIAL, UserCredential, UserCredentialFilter, UserCredentialForCreate, UserCredentialForUpdate,
};

pub struct UserCredentialBmc;
impl DbBmc for UserCredentialBmc {
  const TABLE: &'static str = TABLE_USER_CREDENTIAL;
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
