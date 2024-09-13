use ultimate::Result;
use ultimate_api::v1::{PagePayload, Pagination};

use crate::{
  ctx::CtxW,
  role::role_permission::{RolePermissionBmc, RolePermissionForCreate},
};

use super::{permission_bmc::PermissionBmc, Permission, PermissionFilters, PermissionForCreate, PermissionForUpdate};

pub async fn create(ctx: &CtxW, req: PermissionForCreate) -> Result<i64> {
  let id = PermissionBmc::create(ctx.mm(), req).await?;
  Ok(id)
}

pub async fn find_by_id(ctx: &CtxW, id: i64) -> Result<Permission> {
  let res = PermissionBmc::find_by_id(ctx.mm(), id).await?;
  Ok(res)
}

pub async fn update_by_id(ctx: &CtxW, id: i64, req: PermissionForUpdate) -> Result<()> {
  PermissionBmc::update_by_id(ctx.mm(), id, req).await?;
  Ok(())
}

pub async fn delete_by_id(ctx: &CtxW, id: i64) -> Result<()> {
  PermissionBmc::delete_by_id(ctx.mm(), id).await?;
  Ok(())
}

pub async fn page(ctx: &CtxW, filters: PermissionFilters, pagination: Pagination) -> Result<PagePayload<Permission>> {
  let page = PermissionBmc::page(ctx.mm(), filters, pagination).await?;
  Ok(page)
}

pub async fn find_many(
  ctx: &CtxW,
  filters: PermissionFilters,
  pagination: Option<Pagination>,
) -> Result<Vec<Permission>> {
  let list = PermissionBmc::find_many(ctx.mm(), filters, pagination.map(Into::into)).await?;
  Ok(list)
}

pub async fn assign_roles(ctx: &CtxW, permission_id: i64, role_ids: Vec<i64>) -> Result<()> {
  RolePermissionBmc::insert_many(
    ctx.mm(),
    role_ids.into_iter().map(|role_id| RolePermissionForCreate { permission_id, role_id }).collect(),
  )
  .await?;
  Ok(())
}
