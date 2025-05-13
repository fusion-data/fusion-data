use fusiondata_context::ctx::CtxW;
use ultimate_core::{Result, component::Component, security::pwd::verify_pwd};

use crate::{
  pb::fusion_iam::v1::{SigninRequest, SigninResponse, TokenKind},
  user::{UserFilter, UserSvc},
};

use super::utils::make_token;

#[derive(Clone, Component)]
pub struct AuthSvc {
  #[component]
  user_svc: UserSvc,
}

impl AuthSvc {
  #[tracing::instrument(skip(self, ctx, req))]
  pub async fn signin(&self, ctx: CtxW, req: SigninRequest) -> Result<SigninResponse> {
    let (u, uc) = self.user_svc.get_fetch_credential(&ctx, UserFilter::from(&req)).await?;
    verify_pwd(&req.password, &uc.encrypted_pwd).await?;

    let token = make_token(ctx.app().ultimate_config().security(), u.id)?;
    Ok(SigninResponse { token, token_kind: TokenKind::Bearer as i32 })
  }
}
