use fusiondata_context::ctx::CtxW;
use modelsql::filter::OpValInt64;
use ultimate_core::{component::Component, DataError, Result};

use super::{
  user_role::{UserRoleBmc, UserRoleForCreate},
  User, UserBmc, UserCredential, UserCredentialBmc, UserFilter, UserForCreate, UserForPage, UserForUpdate, UserPage,
};

#[derive(Debug, Clone, Component)]
pub struct UserSvc;

impl UserSvc {
  pub async fn create(&self, ctx: &CtxW, req: UserForCreate) -> Result<i64> {
    let id = UserBmc::create(ctx.mm(), req.validate_and_init()?).await?;
    Ok(id)
  }

  pub async fn page(&self, ctx: &CtxW, req: UserForPage) -> Result<UserPage> {
    let page = UserBmc::page(ctx.mm(), req.filter, req.page.into()).await?;
    Ok(page.into())
  }

  pub async fn find_option_by_id(&self, ctx: &CtxW, id: i64) -> Result<Option<User>> {
    let f = UserFilter { id: Some(OpValInt64::Eq(id).into()), ..Default::default() };
    let u = UserBmc::find_unique(ctx.mm(), vec![f]).await?;
    Ok(u)
  }

  pub async fn find_by_id(&self, ctx: &CtxW, id: i64) -> Result<User> {
    let u = UserBmc::find_by_id(ctx.mm(), id).await?;
    Ok(u)
  }

  pub async fn update_by_id(&self, ctx: &CtxW, id: i64, req: UserForUpdate) -> Result<()> {
    UserBmc::update_by_id(ctx.mm(), id, req).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, ctx: &CtxW, id: i64) -> Result<()> {
    UserBmc::delete_by_id(ctx.mm(), id).await?;
    Ok(())
  }

  #[tracing::instrument(skip(self, ctx, req))]
  pub async fn get_fetch_credential(&self, ctx: &CtxW, req: UserFilter) -> Result<(User, UserCredential)> {
    let u = UserBmc::find_unique(ctx.mm(), vec![req])
      .await?
      .ok_or_else(|| DataError::not_found("User not exists."))?;
    let uc = UserCredentialBmc::find_by_id(ctx.mm(), u.id).await?;
    Ok((u, uc))
  }

  pub async fn assign_role(&self, ctx: &CtxW, user_id: i64, role_ids: Vec<i64>) -> Result<()> {
    let user_roles = role_ids.into_iter().map(|role_id| UserRoleForCreate { user_id, role_id }).collect();
    UserRoleBmc::insert_many(ctx.mm(), user_roles).await?;
    Ok(())
  }
}
