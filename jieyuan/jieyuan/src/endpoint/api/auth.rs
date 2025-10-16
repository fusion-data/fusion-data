use axum::{Json, extract::State, http::request::Parts};
use fusion_core::application::Application;
use fusion_web::{WebResult, extract_ctx, ok_json};
use fusionsql::ModelManager;
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{
  OAuthAuthorizeRequest, OAuthAuthorizeResponse, OAuthTokenRequest, OAuthTokenResponse, RefreshTokenReq, SigninRequest,
  SigninResponse, SignupReq, UserChangeQueryReq, UserChangeQueryResp,
};

use crate::{
  auth::{AuthSvc, OAuthSvc},
  user::UserSvc,
};

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(signin))
    .routes(utoipa_axum::routes!(signup))
    .routes(utoipa_axum::routes!(signout))
    .routes(utoipa_axum::routes!(refresh_token))
    .routes(utoipa_axum::routes!(extract_token))
    .routes(utoipa_axum::routes!(oauth_authorize))
    .routes(utoipa_axum::routes!(oauth_token))
    .routes(utoipa_axum::routes!(users_changes))
}

/// 用户登录
#[utoipa::path(
  post,
  path = "/signin",
  request_body = SigninRequest,
  responses(
    (status = 200, description = "登录成功", body = SigninResponse),
    (status = 401, description = "认证失败")
  ),
  tag = "认证"
)]
async fn signin(State(app): State<Application>, Json(req): Json<SigninRequest>) -> WebResult<SigninResponse> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm.clone());
  let auth_svc = AuthSvc::new(user_svc);
  let response = auth_svc.signin(req).await?;
  ok_json!(response)
}

/// 用户注册
#[utoipa::path(
  post,
  path = "/signup",
  request_body = SignupReq,
  responses(
    (status = 200, description = "注册成功"),
    (status = 400, description = "请求参数错误")
  ),
  tag = "认证"
)]
async fn signup(State(app): State<Application>, Json(req): Json<SignupReq>) -> WebResult<serde_json::Value> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm.clone());
  let auth_svc = AuthSvc::new(user_svc);
  auth_svc.signup(req).await?;
  ok_json!(serde_json::Value::Object(serde_json::Map::new()))
}

/// 用户登出
#[utoipa::path(
  post,
  path = "/signout",
  responses(
    (status = 200, description = "登出成功"),
    (status = 400, description = "请求参数错误")
  ),
  tag = "认证"
)]
async fn signout(parts: Parts, State(app): State<Application>) -> WebResult<serde_json::Value> {
  // 从 Authorization 头中提取 token，简化处理
  let _ctx = extract_ctx(&parts, app.fusion_config().security())?;

  // TODO: 实际实现中应该提取完整的 token 字符串
  // 目前简化处理，直接返回成功
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm.clone());
  let auth_svc = AuthSvc::new(user_svc);
  auth_svc.signout("dummy_token").await?;
  ok_json!(serde_json::Value::Object(serde_json::Map::new()))
}

/// 刷新令牌
#[utoipa::path(
  post,
  path = "/refresh_token",
  request_body = RefreshTokenReq,
  responses(
    (status = 200, description = "刷新成功", body = SigninResponse),
    (status = 401, description = "认证失败")
  ),
  tag = "认证"
)]
async fn refresh_token(State(app): State<Application>, Json(req): Json<RefreshTokenReq>) -> WebResult<SigninResponse> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let user_svc = UserSvc::new(mm.clone());
  let auth_svc = AuthSvc::new(user_svc);
  let response = auth_svc.refresh_token(req).await?;
  ok_json!(response)
}

/// 令牌解析
#[utoipa::path(
  post,
  path = "/extract_token",
  responses(
    (status = 200, description = "解析成功", body = serde_json::Value),
    (status = 401, description = "认证失败")
  ),
  tag = "认证"
)]
async fn extract_token(parts: Parts, State(app): State<Application>) -> WebResult<serde_json::Value> {
  let ctx = extract_ctx(&parts, app.fusion_config().security())?;
  let ctx_json = serde_json::to_value(ctx.payload())?;
  ok_json!(ctx_json)
}

/// OAuth 授权 - 生成授权 URL
#[utoipa::path(
  post,
  path = "/oauth/authorize",
  request_body = OAuthAuthorizeRequest,
  responses(
    (status = 200, description = "授权 URL 生成成功", body = OAuthAuthorizeResponse),
    (status = 400, description = "请求参数错误")
  ),
  tag = "认证"
)]
async fn oauth_authorize(
  State(app): State<Application>,
  Json(req): Json<OAuthAuthorizeRequest>,
) -> WebResult<OAuthAuthorizeResponse> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let oauth_svc = OAuthSvc::new(mm.clone(), app);
  let response = oauth_svc.authorize(req).await?;
  ok_json!(response)
}

/// OAuth 令牌交换 - 使用授权码获取访问令牌
#[utoipa::path(
  post,
  path = "/oauth/token",
  request_body = OAuthTokenRequest,
  responses(
    (status = 200, description = "令牌交换成功", body = OAuthTokenResponse),
    (status = 400, description = "请求参数错误"),
    (status = 401, description = "授权码无效或已过期")
  ),
  tag = "认证"
)]
async fn oauth_token(
  State(app): State<Application>,
  Json(req): Json<OAuthTokenRequest>,
) -> WebResult<OAuthTokenResponse> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let oauth_svc = OAuthSvc::new(mm.clone(), app);
  let response = oauth_svc.exchange_token(req).await?;
  ok_json!(response)
}

/// 用户变更查询 - 用于事件轮询
#[utoipa::path(
  post,
  path = "/users/changes",
  request_body = UserChangeQueryReq,
  responses(
    (status = 200, description = "查询成功", body = UserChangeQueryResp),
    (status = 400, description = "请求参数错误")
  ),
  tag = "认证"
)]
async fn users_changes(
  State(app): State<Application>,
  Json(req): Json<UserChangeQueryReq>,
) -> WebResult<UserChangeQueryResp> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let oauth_svc = OAuthSvc::new(mm.clone(), app);
  let response = oauth_svc.query_user_changes(req).await?;
  ok_json!(response)
}
