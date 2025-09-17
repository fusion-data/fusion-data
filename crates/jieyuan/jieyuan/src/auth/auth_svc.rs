use fusion_core::{Result, application::Application, security::pwd::verify_pwd};

use jieyuan_core::model::{SigninRequest, SigninResponse, TokenType};

use crate::user::UserSvc;

use super::utils::make_token;

#[derive(Clone)]
pub struct AuthSvc {
  user_svc: UserSvc,
}

impl AuthSvc {
  /// 创建新的认证服务实例
  pub fn new(user_svc: UserSvc) -> Self {
    Self { user_svc }
  }

  pub async fn signin(&self, req: SigninRequest) -> Result<SigninResponse> {
    let (filter, password) = req.into_split();
    let (u, uc) = self.user_svc.get_fetch_credential(filter).await?;
    verify_pwd(&password, &uc.encrypted_pwd).await?;

    let config = Application::global().fusion_config();
    let token = make_token(config.security(), u.id)?;
    Ok(SigninResponse { token, token_type: TokenType::Bearer })
  }
}
