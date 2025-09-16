use fusion_core::Result;
use modelsql::{ModelManager, filter::Page, page::PageResult};

use hetuiam_core::types::{CreateRoleDto, Role, RoleFilters, RoleForUpdate, RolePermissionForCreate};

use super::{RolePermissionBmc, bmc::RoleBmc};

#[derive(Debug, Clone)]
pub struct RoleSvc {
  mm: ModelManager,
}

impl RoleSvc {
  /// 创建新的 RoleSvc 实例
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn create(&self, entity_c: CreateRoleDto) -> Result<i64> {
    let role_id = RoleBmc::create(&self.mm, entity_c).await?;
    Ok(role_id)
  }

  pub async fn find_option_by_id(&self, id: i64) -> Result<Option<Role>> {
    let r = RoleBmc::find_by_id(&self.mm, id).await.ok();
    Ok(r)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<Role> {
    let r = RoleBmc::find_by_id(&self.mm, id).await?;
    Ok(r)
  }

  pub async fn update_by_id(&self, id: i64, entity_u: RoleForUpdate) -> Result<()> {
    RoleBmc::update_by_id(&self.mm, id, entity_u).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, id: i64) -> Result<()> {
    RoleBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  pub async fn page(&self, filters: RoleFilters) -> Result<PageResult<Role>> {
    let pagination = Page::default();
    let page = RoleBmc::page(&self.mm, filters, pagination).await?;
    Ok(page)
  }

  pub async fn assign_permissions(&self, role_id: i64, permission_ids: Vec<i64>) -> Result<()> {
    RolePermissionBmc::insert_many(
      &self.mm,
      permission_ids
        .into_iter()
        .map(|permission_id| RolePermissionForCreate { role_id, permission_id })
        .collect(),
    )
    .await?;
    Ok(())
  }
}
