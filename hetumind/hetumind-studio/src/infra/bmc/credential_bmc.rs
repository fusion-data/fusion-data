use std::sync::OnceLock;

use fusionsql::{
  base::{BmcConfig, DbBmc},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use crate::domain::credential::{CredentialEntity, CredentialFilter, CredentialForInsert, CredentialForUpdate};

pub struct CredentialBmc;
impl DbBmc for CredentialBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("credential_entity").with_use_logical_deletion(true))
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
