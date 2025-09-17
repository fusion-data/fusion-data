use fusion_core::Result;
use modelsql::{ModelManager, page::PageResult};

use jieyuan_core::types::{
  Permission, PermissionForCreate, PermissionForPage, PermissionForUpdate, RolePermissionForCreate,
};

use crate::role::RolePermissionBmc;

use super::PermissionBmc;

#[derive(Clone)]
pub struct PermissionSvc {
  mm: ModelManager,
}

impl PermissionSvc {
  /// 创建新的 PermissionSvc 实例
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn create(&self, req: PermissionForCreate) -> Result<i64> {
    let id = PermissionBmc::create(&self.mm, req).await?;
    Ok(id)
  }

  pub async fn find_option_by_id(&self, id: i64) -> Result<Option<Permission>> {
    let res = PermissionBmc::find_by_id(&self.mm, id).await.ok();
    Ok(res)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<Permission> {
    let res = PermissionBmc::find_by_id(&self.mm, id).await?;
    Ok(res)
  }

  pub async fn update_by_id(&self, id: i64, req: PermissionForUpdate) -> Result<()> {
    PermissionBmc::update_by_id(&self.mm, id, req).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, id: i64) -> Result<()> {
    PermissionBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  pub async fn page(&self, req: PermissionForPage) -> Result<PageResult<Permission>> {
    let page = PermissionBmc::page(&self.mm, req.filters, req.page).await?;
    Ok(page)
  }

  pub async fn find_many(&self, req: PermissionForPage) -> Result<Vec<Permission>> {
    let list = PermissionBmc::find_many(&self.mm, req.filters, Some(req.page)).await?;
    Ok(list)
  }

  pub async fn assign_roles(&self, permission_id: i64, role_ids: Vec<i64>) -> Result<()> {
    RolePermissionBmc::insert_many(
      &self.mm,
      role_ids.into_iter().map(|role_id| RolePermissionForCreate { permission_id, role_id }).collect(),
    )
    .await?;
    Ok(())
  }
}
