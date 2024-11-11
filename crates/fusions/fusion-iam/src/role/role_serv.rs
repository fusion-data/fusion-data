use fusiondata_context::ctx::CtxW;
use ultimate::Result;
use ultimate_api::v1::{PagePayload, Pagination};

use crate::{pb::fusion_iam::v1::CreateRoleDto, role::role_bmc::RoleBmc};

use super::{
  role_permission::{RolePermissionBmc, RolePermissionForCreate},
  Role, RoleFilters, RoleForUpdate,
};

pub async fn create(ctx: &CtxW, entity_c: CreateRoleDto, permission_ids: Vec<i64>) -> Result<i64> {
  let mm = ctx.mm();

  let role_id = RoleBmc::create(mm, entity_c).await?;

  if !permission_ids.is_empty() {
    let data =
      permission_ids.into_iter().map(|permission_id| RolePermissionForCreate { role_id, permission_id }).collect();
    RolePermissionBmc::insert_many(mm, data).await.unwrap();
  }

  Ok(role_id)
}

pub async fn find_by_id(ctx: &CtxW, id: i64) -> Result<Role> {
  let r = RoleBmc::find_by_id(ctx.mm(), id).await?;
  Ok(r)
}

pub async fn update_by_id(ctx: &CtxW, id: i64, entity_u: RoleForUpdate) -> Result<()> {
  RoleBmc::update_by_id(ctx.mm(), id, entity_u).await?;
  Ok(())
}

pub async fn delete_by_id(ctx: &CtxW, id: i64) -> Result<()> {
  RoleBmc::delete_by_id(ctx.mm(), id).await?;
  Ok(())
}

pub async fn page(ctx: &CtxW, filters: RoleFilters, pagination: Pagination) -> Result<PagePayload<Role>> {
  let page = RoleBmc::page(ctx.mm(), filters, pagination).await?;
  Ok(page)
}

pub async fn assign_permissions(ctx: &CtxW, role_id: i64, permission_ids: Vec<i64>) -> Result<()> {
  RolePermissionBmc::insert_many(
    ctx.mm(),
    permission_ids.into_iter().map(|permission_id| RolePermissionForCreate { role_id, permission_id }).collect(),
  )
  .await?;
  Ok(())
}
