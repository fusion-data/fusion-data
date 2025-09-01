use hetumind_core::credential::TokenType;
use serde::{Deserialize, Serialize};
use fusion_common::regex::{is_email, is_phone};

#[derive(Deserialize)]
pub struct SigninRequest {
  pub account: String,
  pub password: String,
}

impl SigninRequest {
  pub fn as_email(&self) -> Option<&str> {
    if is_email(&self.account) { Some(&self.account) } else { None }
  }

  pub fn as_phone(&self) -> Option<&str> {
    if is_phone(&self.account) { Some(&self.account) } else { None }
  }
}

#[derive(Serialize)]
pub struct SigninResponse {
  pub token: String,
  pub token_type: TokenType,
}

#[derive(Deserialize)]
pub struct SignupRequest {
  pub email: String,
  pub password: String,
}
