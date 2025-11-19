use std::sync::OnceLock;

use fusionsql::{
  base::{BmcConfig, DbBmc},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use jieyuan_core::model::{
  PolicyAttachmentEntity, PolicyAttachmentFilter, PolicyAttachmentForCreate, PolicyAttachmentForUpdate,
  TABLE_POLICY_ATTACHMENT,
};

#[allow(dead_code)]
pub struct PolicyAttachmentBmc;

impl DbBmc for PolicyAttachmentBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_POLICY_ATTACHMENT))
  }
}

generate_pg_bmc_common!(
  Bmc: PolicyAttachmentBmc,
  Entity: PolicyAttachmentEntity,
  ForCreate: PolicyAttachmentForCreate,
  ForUpdate: PolicyAttachmentForUpdate,
  ForInsert: PolicyAttachmentForCreate,
);

generate_pg_bmc_filter!(
  Bmc: PolicyAttachmentBmc,
  Entity: PolicyAttachmentEntity,
  Filter: PolicyAttachmentFilter,
);
