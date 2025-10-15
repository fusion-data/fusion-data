use axum::extract::FromRequestParts;
use fusion_common::time::now_utc;
use fusion_core::{
  DataError,
  application::Application,
  configuration::KeyConf,
};
use fusion_web::WebError;
use hetumind_context::utils::{make_token, verify_token};
use hetumind_core::credential::TokenType;
use http::request::Parts;
use log::info;

use super::{InvalidAuthTokenBmc, RefreshTokenRequest, RefreshTokenResponse, SignoutRequest};
use crate::domain::user::{UserBmc, UserStatus};

#[derive(Clone)]
pub struct SignSvc {
  mm: fusionsql::ModelManager,
  application: Application,
}

impl SignSvc {
  /// 验证来自 Jieyuan 的令牌并返回本地令牌（用于代理模式）
  pub async fn verify_and_proxy_token(&self, jieyuan_token: &str) -> Result<super::SigninResponse, DataError> {
    // TODO: 实现 Jieyuan 令牌验证逻辑，可能需要调用 Jieyuan 的验证端点或使用 JWKS
    // 这里暂时返回未实现错误，在真实实现中需要：
    // 1. 验证 Jieyuan 令牌签名
    // 2. 解析令牌获取用户信息
    // 3. 确保用户在本地存在
    Err(DataError::unauthorized("Jieyuan token verification not yet implemented"))
  }

  /// 刷新访问令牌
  pub async fn refresh_token(&self, refresh_req: RefreshTokenRequest) -> Result<RefreshTokenResponse, DataError> {
    // 验证刷新令牌
    let payload = verify_token(&refresh_req.refresh_token, self.application.fusion_config().security().pwd())?;
    let user_id = payload
      .get_subject()
      .ok_or_else(|| DataError::unauthorized("Invalid refresh token: missing user id"))?;

    // 验证用户是否存在且状态正常
    let user = UserBmc::get_by_id(
      &self.mm,
      user_id.parse::<i64>().map_err(|_| DataError::unauthorized("Invalid user id in token"))?,
    )
    .await
    .map_err(DataError::from)?
    .ok_or(DataError::unauthorized("User not found"))?;

    if user.status != UserStatus::Enabled {
      return Err(DataError::unauthorized("User account is disabled"));
    }

    // 生成新的访问令牌
    let access_token = make_token(user.id.to_string(), self.application.fusion_config().security().pwd())?;
    let expires_in = self.application.fusion_config().security().pwd().expires_in();

    Ok(RefreshTokenResponse { access_token, token_type: TokenType::Bearer, expires_in })
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
    Ok(SignSvc {
      mm: state.component(),
      application: state.clone()
    })
  }
}