use axum::{Router, routing::post};
use fusion_core::application::Application;

use crate::domain::auth::{SignSvc, SigninRequest, SigninResponse, SignupRequest, RefreshTokenRequest, RefreshTokenResponse, SignoutRequest};
use fusion_web::{WebResult, extract::JsonOrForm, ok_json};

pub fn auth_routes() -> Router<Application> {
  Router::new()
    .route("/signin", post(signin))
    .route("/signup", post(signup))
    .route("/refresh", post(refresh_token))
    .route("/signout", post(signout))
}

async fn signin(sign_svc: SignSvc, JsonOrForm(signin_req): JsonOrForm<SigninRequest>) -> WebResult<SigninResponse> {
  let signin_resp = sign_svc.signin(signin_req).await?;
  ok_json!(signin_resp)
}

async fn signup(sign_svc: SignSvc, JsonOrForm(signup_req): JsonOrForm<SignupRequest>) -> WebResult<()> {
  sign_svc.signup(signup_req).await?;
  ok_json!()
}

async fn refresh_token(sign_svc: SignSvc, JsonOrForm(refresh_req): JsonOrForm<RefreshTokenRequest>) -> WebResult<RefreshTokenResponse> {
  let refresh_resp = sign_svc.refresh_token(refresh_req).await?;
  ok_json!(refresh_resp)
}

async fn signout(sign_svc: SignSvc, JsonOrForm(signout_req): JsonOrForm<SignoutRequest>) -> WebResult<()> {
  sign_svc.signout(signout_req).await?;
  ok_json!()
}
