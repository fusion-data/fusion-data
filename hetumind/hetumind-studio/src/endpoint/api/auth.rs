use axum::{Router, routing::post};
use fusion_core::application::Application;

use crate::domain::auth::{SignSvc, SigninRequest, SigninResponse, SignupRequest};
use fusion_web::{WebResult, extract::JsonOrForm, ok_json};

pub fn auth_routes() -> Router<Application> {
  Router::new().route("/signin", post(signin)).route("/signup", post(signup))
}

async fn signin(sign_svc: SignSvc, JsonOrForm(signin_req): JsonOrForm<SigninRequest>) -> WebResult<SigninResponse> {
  let signin_resp = sign_svc.signin(signin_req).await?;
  ok_json!(signin_resp)
}

async fn signup(sign_svc: SignSvc, JsonOrForm(signup_req): JsonOrForm<SignupRequest>) -> WebResult<()> {
  sign_svc.signup(signup_req).await?;
  ok_json!()
}
