use axum::{Json, extract::State};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use fusionsql::ModelManager;
use utoipa_axum::router::OpenApiRouter;

use jieyuan_core::model::{OAuthAuthorizeRequest, OAuthAuthorizeResponse, OAuthTokenRequest, OAuthTokenResponse};

use crate::auth::OAuthSvc;

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new()
    .routes(utoipa_axum::routes!(oauth_authorize))
    .routes(utoipa_axum::routes!(oauth_token))
}

/// OAuth 授权 - 生成授权 URL
#[utoipa::path(
  post,
  path = "/authorize",
  request_body = OAuthAuthorizeRequest,
  responses(
    (status = 200, description = "授权 URL 生成成功", body = OAuthAuthorizeResponse),
    (status = 400, description = "请求参数错误")
  ),
  tag = "OAuth"
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
  path = "/token",
  request_body = OAuthTokenRequest,
  responses(
    (status = 200, description = "令牌交换成功", body = OAuthTokenResponse),
    (status = 400, description = "请求参数错误"),
    (status = 401, description = "授权码无效或已过期")
  ),
  tag = "OAuth"
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
