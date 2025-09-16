use fusion_core::Result;
use modelsql::{ModelManager, filter::Page, page::PageResult};

use hetuiam_core::types::{
  Permission, PermissionFilters, PermissionForCreate, PermissionForUpdate, RolePermissionForCreate,
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

  pub async fn page(&self, filters: PermissionFilters) -> Result<PageResult<Permission>> {
    let pagination = Page::default();
    let page = PermissionBmc::page(&self.mm, filters, pagination).await?;
    Ok(page)
  }

  pub async fn find_many(&self, filters: PermissionFilters, pagination: Option<Page>) -> Result<Vec<Permission>> {
    let list = PermissionBmc::find_many(&self.mm, filters, pagination.map(Into::into)).await?;
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
