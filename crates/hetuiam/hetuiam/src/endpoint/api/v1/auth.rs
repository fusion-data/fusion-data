use axum::{Json, extract::State};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use modelsql::ModelManager;
use utoipa_axum::router::OpenApiRouter;

use hetuiam_core::types::{SigninRequest, SigninResponse};

use crate::{auth::AuthSvc, user::UserSvc};

pub fn routes() -> OpenApiRouter<Application> {
  OpenApiRouter::new().routes(utoipa_axum::routes!(signin))
}

/// 用户登录
#[utoipa::path(
  post,
  path = "/auth/signin",
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
