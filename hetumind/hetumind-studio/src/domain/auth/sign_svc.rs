use axum::extract::FromRequestParts;
use fusion_common::time::{OffsetDateTime, now_utc};
use fusion_core::{
  DataError,
  application::Application,
  configuration::KeyConf,
  security::pwd::{generate_pwd, verify_pwd},
};
use fusion_web::WebError;
use fusionsql::{ModelManager, filter::OpValString};
use hetumind_context::utils::{make_token, make_refresh_token, verify_token};
use hetumind_core::credential::TokenType;
use http::request::Parts;
use log::info;

use crate::domain::user::{UserBmc, UserFilter, UserForCreate, UserStatus};
use super::{InvalidAuthTokenBmc, SigninRequest, SigninResponse, SignupRequest, RefreshTokenRequest, RefreshTokenResponse, SignoutRequest};

#[derive(Clone)]
pub struct SignSvc {
  mm: ModelManager,
  application: Application,
}

impl SignSvc {
  pub async fn signin(&self, signin_req: SigninRequest) -> Result<SigninResponse, DataError> {
    let email = signin_req.as_email();
    let phone = signin_req.as_phone();

    let filter = if let Some(email) = email {
      UserFilter { email: Some(OpValString::eq(email)), ..Default::default() }
    } else if let Some(phone) = phone {
      UserFilter { phone: Some(OpValString::eq(phone)), ..Default::default() }
    } else {
      return Err(DataError::unauthorized("Parameter account must be email or phone"));
    };

    let user = UserBmc::find_unique(&self.mm, vec![filter])
      .await?
      .ok_or(DataError::unauthorized("User not found"))?;

    let password = user.password.ok_or_else(|| DataError::unauthorized("User password not set"))?;
    verify_pwd(&signin_req.password, &password).await?;

    let access_token = make_token(user.id.to_string(), self.application.fusion_config().security().pwd())?;
    let refresh_token = make_refresh_token(user.id.to_string(), self.application.fusion_config().security().pwd())?;
    let expires_in = self.application.fusion_config().security().pwd().expires_at().timestamp() - now_utc().timestamp();

    Ok(SigninResponse {
      access_token,
      refresh_token,
      token_type: TokenType::Bearer,
      expires_in,
    })
  }

  pub async fn signup(&self, signup_req: SignupRequest) -> Result<(), DataError> {
    let password = generate_pwd(&signup_req.password).await?;
    let entity_c =
      UserForCreate { email: signup_req.email, phone: None, name: None, password, status: UserStatus::Enabled };

    let user_id = UserBmc::create(&self.mm, entity_c).await?;
    info!("User signup success: {}", user_id);
    Ok(())
  }

  /// 刷新访问令牌
  pub async fn refresh_token(&self, refresh_req: RefreshTokenRequest) -> Result<RefreshTokenResponse, DataError> {
    // 验证刷新令牌
    let payload = verify_token(&refresh_req.refresh_token, self.application.fusion_config().security().pwd())?;
    let user_id = payload.get_subject().ok_or_else(|| DataError::unauthorized("Invalid refresh token: missing user id"))?;

    // 验证用户是否存在且状态正常
    let user = UserBmc::get_by_id(&self.mm, user_id.parse::<i64>()
      .map_err(|_| DataError::unauthorized("Invalid user id in token"))?)
      .await
      .map_err(DataError::from)?
      .ok_or(DataError::unauthorized("User not found"))?;

    if user.status != UserStatus::Enabled {
      return Err(DataError::unauthorized("User account is disabled"));
    }

    // 生成新的访问令牌
    let access_token = make_token(user.id.to_string(), self.application.fusion_config().security().pwd())?;
    let expires_in = self.application.fusion_config().security().pwd().expires_at().timestamp() - now_utc().timestamp();

    Ok(RefreshTokenResponse {
      access_token,
      token_type: TokenType::Bearer,
      expires_in,
    })
  }

  /// 登出并加入令牌黑名单
  pub async fn signout(&self, signout_req: SignoutRequest) -> Result<(), DataError> {
    let token_to_invalidate = signout_req.token.unwrap_or_default();

    if !token_to_invalidate.is_empty() {
      // 将指定的令牌加入黑名单
      if let Ok(payload) = verify_token(&token_to_invalidate, self.application.fusion_config().security().pwd()) {
        let exp_timestamp = payload.get_exp().unwrap_or_else(|| {
          // 如果没有过期时间，设置24小时后过期
          (now_utc().timestamp() + 24 * 60 * 60) as i64
        });
        let expires_at = fusion_common::time::datetime_from_millis(exp_timestamp * 1000);

        InvalidAuthTokenBmc::add_token(&self.mm, &token_to_invalidate, expires_at).await?;
        info!("Token added to blacklist: {}", &token_to_invalidate[..8.min(token_to_invalidate.len())]);
      }
    }

    Ok(())
  }
}

impl FromRequestParts<Application> for SignSvc {
  type Rejection = WebError;

  async fn from_request_parts(_parts: &mut Parts, state: &Application) -> Result<Self, Self::Rejection> {
    Ok(SignSvc { mm: state.component(), application: state.clone() })
  }
}
