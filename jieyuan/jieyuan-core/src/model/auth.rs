use fusionsql_core::filter::OpValString;
use serde::{Deserialize, Serialize};

use super::UserFilter;

/// Token kind enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i32)]
pub enum TokenType {
  Unspecified = 0,
  #[default]
  Bearer = 1,
}

impl From<i32> for TokenType {
  fn from(value: i32) -> Self {
    match value {
      1 => TokenType::Bearer,
      _ => TokenType::Unspecified,
    }
  }
}

/// Signin request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SigninRequest {
  pub email: Option<String>,
  pub phone: Option<String>,
  pub password: String,
}

impl SigninRequest {
  pub fn into_split(self) -> (UserFilter, String) {
    (
      UserFilter {
        email: self.email.map(OpValString::eq),
        phone: self.phone.map(OpValString::eq),
        ..Default::default()
      },
      self.password,
    )
  }
}

/// Signin response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SigninResponse {
  pub token: String,
  pub token_type: TokenType,
}
