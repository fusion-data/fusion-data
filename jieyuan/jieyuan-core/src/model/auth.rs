use fusion_common::page::{Page, Paged};
use fusionsql_core::filter::{OpValDateTime, OpValString};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::AsRefStr;

use super::{UserFilter, UserStatus};

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

/// Signup request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SignupReq {
  pub email: Option<String>,
  pub phone: Option<String>,
  pub password: String,
}

impl SignupReq {
  pub fn validate(&self) -> Result<(), String> {
    // 验证密码不为空
    if self.password.is_empty() {
      return Err("password is required".to_string());
    }

    // 验证 email 和 phone 互斥且至少提供一个
    match (&self.email, &self.phone) {
      (Some(_), Some(_)) => Err("email and phone cannot be provided simultaneously".to_string()),
      (None, None) => Err("either email or phone must be provided".to_string()),
      _ => Ok(()),
    }
  }

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

/// Refresh token request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct RefreshTokenReq {
  pub refresh_token: String,
}

/// Signin request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SigninRequest {
  pub email: Option<String>,
  pub phone: Option<String>,
  pub password: String,
  pub tenant_id: Option<i64>,
}

impl SigninRequest {
  pub fn into_split(self) -> (UserFilter, String, Option<i64>) {
    (
      UserFilter {
        email: self.email.map(OpValString::eq),
        phone: self.phone.map(OpValString::eq),
        ..Default::default()
      },
      self.password,
      self.tenant_id,
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

/// OAuth Provider enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize_repr, Deserialize_repr, AsRefStr)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[strum(serialize_all = "snake_case")]
#[repr(i32)]
pub enum OAuthProvider {
  Unspecified = 0,
  #[default]
  Wechat = 1,
  Alipay = 2,
  Weibo = 3,
  Gitee = 4,
  Github = 5,
}

impl From<i32> for OAuthProvider {
  fn from(value: i32) -> Self {
    match value {
      1 => OAuthProvider::Wechat,
      2 => OAuthProvider::Alipay,
      3 => OAuthProvider::Weibo,
      4 => OAuthProvider::Gitee,
      5 => OAuthProvider::Github,
      _ => OAuthProvider::Unspecified,
    }
  }
}

/// OAuth authorize request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct OAuthAuthorizeRequest {
  pub provider: OAuthProvider,
  pub redirect_uri: Option<String>,
  pub state: Option<String>,
  pub code_challenge: Option<String>,
  pub code_challenge_method: Option<String>,
}

/// OAuth authorize response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct OAuthAuthorizeResponse {
  pub authorize_url: String,
  pub state: Option<String>,
  pub code_verifier: Option<String>,
}

/// OAuth token exchange request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct OAuthTokenRequest {
  pub provider: OAuthProvider,
  pub code: String,
  pub code_verifier: Option<String>,
  pub redirect_uri: Option<String>,
}

/// OAuth token exchange response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct OAuthTokenResponse {
  pub token: String,
  pub token_type: TokenType,
  pub expires_in: Option<i64>,
  pub refresh_token: Option<String>,
  pub iam_user_id: Option<i64>,
  pub subject: Option<String>,
}

/// User change query request for polling
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserChangeQueryReq {
  pub page: Page,
  pub filters: Vec<UserChangeFilter>,
}

/// Filter for user changes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserChangeFilter {
  #[serde(rename = "updated_at")]
  pub updated_at: Option<OpValDateTime>,
}

/// User change query response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserChangeQueryResp {
  pub page: Paged,
  pub result: Vec<UserChangeInfo>,
}

/// User change information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields))]
pub struct UserChangeInfo {
  pub id: i64,
  pub status: UserStatus,
  pub updated_at: String,
}

/// Tenant user change query request for polling
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserChangeQueryReq {
  pub page: Page,
  pub filters: Vec<TenantUserChangeFilter>,
}

/// Filter for tenant user changes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserChangeFilter {
  #[serde(rename = "updated_at")]
  pub updated_at: Option<OpValDateTime>,
}

/// Tenant user change query response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserChangeQueryResp {
  pub page: Paged,
  pub result: Vec<TenantUserChangeInfo>,
}

/// Tenant user change information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields))]
pub struct TenantUserChangeInfo {
  pub tenant_id: i64,
  pub user_id: i64,
  pub status: i16,
  pub updated_at: String,
}
