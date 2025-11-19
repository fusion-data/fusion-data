use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{self, BmcConfig, DbBmc},
};

use jieyuan_core::model::{RolePermissionForCreate, TABLE_ROLE_PERMISSION};

pub struct RolePermissionBmc;
impl DbBmc for RolePermissionBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_ROLE_PERMISSION).with_has_updated_at(false))
  }
}

impl RolePermissionBmc {
  pub async fn insert_many(mm: &ModelManager, data: Vec<RolePermissionForCreate>) -> Result<u64, SqlError> {
    base::insert_many::<Self, _>(mm, data).await
  }
}
