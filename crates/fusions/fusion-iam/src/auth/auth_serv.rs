use fusiondata_context::ctx::CtxW;
use ultimate::{security::pwd::verify_pwd, Result};

use crate::{
  pb::fusion_iam::v1::{SigninReplay, SigninRequest, TokenKind},
  user::{user_serv, UserFilter},
};

use super::utils::make_token;

#[tracing::instrument(skip(ctx, req))]
pub async fn signin(ctx: CtxW, req: SigninRequest) -> Result<SigninReplay> {
  let (u, uc) = user_serv::get_fetch_credential(&ctx, UserFilter::from(&req)).await?;
  verify_pwd(&req.password, &uc.encrypted_pwd).await?;

  let token = make_token(ctx.app().ultimate_config().security(), u.id)?;
  Ok(SigninReplay { token, token_kind: TokenKind::Bearer as i32 })
}
