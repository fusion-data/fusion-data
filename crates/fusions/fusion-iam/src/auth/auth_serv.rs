use fusion_server::app::AppState;
use ultimate::{security::pwd::verify_pwd, Result};

use crate::{
  pb::fusion_iam::v1::{SigninReplay, SigninRequest, TokenKind},
  user::{user_serv, UserFilter},
};

use super::utils::make_token;

pub async fn signin(app: &AppState, req: SigninRequest) -> Result<SigninReplay> {
  let ctx = app.create_super_admin_ctx();

  let (u, uc) = user_serv::get_fetch_credential(&ctx, UserFilter::from(&req)).await?;
  verify_pwd(&req.password, &uc.encrypted_pwd).await?;

  let token = make_token(app.configuration().security(), u.id)?;
  Ok(SigninReplay { token, token_kind: TokenKind::Bearer as i32 })
}
