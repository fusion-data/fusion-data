use fusion_common::time::OffsetDateTime;
use modelsql_core::filter::{OpValsDateTime, OpValsInt32, OpValsInt64, OpValsString, Page};
use serde::{Deserialize, Serialize};

/// User status enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i32)]
pub enum UserStatus {
  #[default]
  Unspecified = 0,
  Disabled = 99,
  Enabled = 100,
}

impl From<i32> for UserStatus {
  fn from(value: i32) -> Self {
    match value {
      99 => UserStatus::Disabled,
      100 => UserStatus::Enabled,
      _ => UserStatus::Unspecified,
    }
  }
}

/// Gender enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i32)]
pub enum Gender {
  #[default]
  Unspecified = 0,
  Male = 1,
  Female = 2,
}

impl From<i32> for Gender {
  fn from(value: i32) -> Self {
    match value {
      1 => Gender::Male,
      2 => Gender::Female,
      _ => Gender::Unspecified,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, modelsql::field::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct User {
  pub id: i64,
  pub email: Option<String>,
  pub phone: Option<String>,
  pub name: String,
  pub status: UserStatus,
  pub gender: Gender,
  pub created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub updated_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserForCreate {
  pub email: Option<String>,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub status: Option<UserStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserForUpdate {
  pub name: Option<String>,
  pub status: Option<UserStatus>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserForPage {
  pub page: Page,
  pub filter: Vec<UserFilter>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserFilter {
  pub id: Option<OpValsInt64>,

  pub email: Option<OpValsString>,

  pub phone: Option<OpValsString>,

  pub name: Option<OpValsString>,

  pub status: Option<OpValsInt32>,

  pub gender: Option<OpValsInt32>,

  pub created_by: Option<OpValsInt64>,

  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: Option<OpValsDateTime>,

  pub updated_by: Option<OpValsInt64>,

  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub updated_at: Option<OpValsDateTime>,
}

#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, modelsql::field::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserCredential {
  pub id: i64,
  pub encrypted_pwd: String,
  pub created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub updated_at: Option<OffsetDateTime>,
}

#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
pub struct UserCredentialForCreate {
  pub id: i64,
  pub encrypted_pwd: String,
}

#[derive(Default)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
pub struct UserCredentialForUpdate {
  pub id: Option<i64>,
  pub encrypted_pwd: Option<String>,
}

#[derive(Default)]
#[cfg_attr(feature = "with-db", derive(modelsql::filter::FilterNodes))]
pub struct UserCredentialFilter {
  pub id: Option<OpValsInt64>,

  pub created_by: Option<OpValsInt64>,

  pub created_at: Option<OpValsDateTime>,

  pub updated_by: Option<OpValsInt64>,

  pub updated_at: Option<OpValsDateTime>,
}
