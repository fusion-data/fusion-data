use modelsql::{
  ModelManager, SqlError,
  base::{self, DbBmc},
};

use hetuiam_core::{infra::tables::TABLE_ROLE_PERMISSION, types::RolePermissionForCreate};

pub struct RolePermissionBmc;
impl DbBmc for RolePermissionBmc {
  const TABLE: &'static str = TABLE_ROLE_PERMISSION;

  fn _has_modification_timestamps() -> bool {
    false
  }
}

impl RolePermissionBmc {
  pub async fn insert_many(mm: &ModelManager, data: Vec<RolePermissionForCreate>) -> Result<u64, SqlError> {
    base::insert_many::<Self, _>(mm, data).await
  }
}
