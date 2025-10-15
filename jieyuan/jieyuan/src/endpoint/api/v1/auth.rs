use axum::{Json, extract::State};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use fusionsql::ModelManager;
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{
  SigninRequest, SigninResponse, OAuthAuthorizeRequest, OAuthAuthorizeResponse,
  OAuthTokenRequest, OAuthTokenResponse, UserChangeQueryReq, UserChangeQueryResp
};

use crate::{auth::{AuthSvc, OAuthSvc}, user::UserSvc};

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(signin))
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
  Json(req): Json<OAuthAuthorizeRequest>
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
  Json(req): Json<OAuthTokenRequest>
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
  Json(req): Json<UserChangeQueryReq>
) -> WebResult<UserChangeQueryResp> {
  let mm = app.get_component::<ModelManager>().unwrap();
  let oauth_svc = OAuthSvc::new(mm.clone(), app);
  let response = oauth_svc.query_user_changes(req).await?;
  ok_json!(response)
}
