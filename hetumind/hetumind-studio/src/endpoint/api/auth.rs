use axum::{
  Router,
  extract::Query,
  response::Redirect,
  routing::{get, post},
};
use fusion_core::application::Application;
use serde::Deserialize;

use crate::domain::auth::{OAuthProxySvc, RefreshTokenRequest, RefreshTokenResponse, SignSvc, SignoutRequest};
use fusion_web::{WebResult, extract::JsonOrForm, ok_json};

#[derive(Debug, Deserialize)]
struct RedirectQuery {
  redirect_to: Option<String>,
}

pub fn auth_routes() -> Router<Application> {
  Router::new()
    .route("/signin", get(signin_redirect))
    .route("/signup", get(signup_redirect))
    .route("/oauth/{provider}/start", get(oauth_redirect))
    .route("/token/exchange", post(token_exchange))
    .route("/refresh", post(refresh_token))
    .route("/signout", post(signout))
}

/// 重定向到 Jieyuan 登录页面
async fn signin_redirect(oauth_proxy_svc: OAuthProxySvc, Query(query): Query<RedirectQuery>) -> Redirect {
  let signin_url = oauth_proxy_svc.get_signin_url(query.redirect_to.as_deref());
  Redirect::temporary(&signin_url)
}

/// 重定向到 Jieyuan 注册页面
async fn signup_redirect(oauth_proxy_svc: OAuthProxySvc, Query(query): Query<RedirectQuery>) -> Redirect {
  let signup_url = oauth_proxy_svc.get_signup_url(query.redirect_to.as_deref());
  Redirect::temporary(&signup_url)
}

/// 重定向到 Jieyuan OAuth 授权页面
async fn oauth_redirect(
  oauth_proxy_svc: OAuthProxySvc,
  axum::extract::Path(provider): axum::extract::Path<String>,
  Query(query): Query<RedirectQuery>,
) -> Redirect {
  let oauth_url = oauth_proxy_svc.get_oauth_url(&provider, query.redirect_to.as_deref());
  Redirect::temporary(&oauth_url)
}

/// 令牌交换 - 验证 Jieyuan 令牌并返回本地令牌
#[derive(Debug, Deserialize)]
struct TokenExchangeRequest {
  jieyuan_token: String,
}

#[derive(Debug, serde::Serialize)]
struct TokenExchangeResponse {
  access_token: String,
  refresh_token: String,
  token_type: String,
  expires_in: i64,
}

async fn token_exchange(
  sign_svc: SignSvc,
  JsonOrForm(req): JsonOrForm<TokenExchangeRequest>,
) -> WebResult<TokenExchangeResponse> {
  let signin_resp = sign_svc.verify_and_proxy_token(&req.jieyuan_token).await?;

  Ok(
    TokenExchangeResponse {
      access_token: signin_resp.access_token,
      refresh_token: signin_resp.refresh_token.unwrap_or_default(),
      token_type: "Bearer".to_string(),
      expires_in: signin_resp.expires_in,
    }
    .into(),
  )
}

async fn refresh_token(
  sign_svc: SignSvc,
  JsonOrForm(refresh_req): JsonOrForm<RefreshTokenRequest>,
) -> WebResult<RefreshTokenResponse> {
  let refresh_resp = sign_svc.refresh_token(refresh_req).await?;
  ok_json!(refresh_resp)
}

async fn signout(sign_svc: SignSvc, JsonOrForm(signout_req): JsonOrForm<SignoutRequest>) -> WebResult<()> {
  sign_svc.signout(signout_req).await?;
  ok_json!()
}
