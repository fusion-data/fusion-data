use hetumind_core::credential::TokenType;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SigninResponse {
  pub access_token: String,
  pub refresh_token: Option<String>,
  pub token_type: TokenType,
  pub expires_in: i64, // seconds
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
  pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshTokenResponse {
  pub access_token: String,
  pub token_type: TokenType,
  pub expires_in: i64, // seconds
}

#[derive(Deserialize)]
pub struct SignoutRequest {
  pub token: Option<String>, // optional, if not provided, use current authenticated token
}
