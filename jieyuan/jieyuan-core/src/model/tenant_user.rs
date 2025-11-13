use fusion_common::time::{DateTime, FixedOffset};
use fusionsql_core::filter::{OpValDateTime, OpValInt64};
use fusionsql_core::page::Page;
use serde::{Deserialize, Serialize};

/// Tenant User status enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[repr(i16)]
pub enum TenantUserStatus {
  #[default]
  Disabled = 99,
  Active = 100,
}

impl From<i16> for TenantUserStatus {
  fn from(value: i16) -> Self {
    match value {
      100 => TenantUserStatus::Active,
      _ => TenantUserStatus::Disabled,
    }
  }
}

#[cfg(feature = "with-db")]
impl From<TenantUserStatus> for sea_query::Value {
  fn from(status: TenantUserStatus) -> Self {
    (status as i16).into()
  }
}

#[cfg(feature = "with-db")]
impl sea_query::Nullable for TenantUserStatus {
  fn null() -> sea_query::Value {
    sea_query::Value::SmallInt(None)
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields), sea_query::enum_def)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUser {
  pub tenant_id: i64,
  pub user_id: i64,
  pub status: TenantUserStatus,
  pub created_at: DateTime<FixedOffset>,
  pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserForCreate {
  pub tenant_id: i64,
  pub user_id: i64,
  pub status: Option<TenantUserStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserForUpdate {
  pub status: Option<TenantUserStatus>,
}

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserForPage {
  pub page: Page,
  pub filters: Vec<TenantUserFilter>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::filter::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TenantUserFilter {
  pub tenant_id: Option<OpValInt64>,
  pub user_id: Option<OpValInt64>,
  pub status: Option<OpValInt64>,
  pub created_at: Option<OpValDateTime>,
  pub updated_at: Option<OpValDateTime>,
}

/// User with tenant information for login and authorization
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct UserWithTenant {
  pub id: i64,
  pub email: Option<String>,
  pub phone: Option<String>,
  pub name: String,
  pub status: super::user::UserStatus,
  pub gender: super::user::Gender,
  pub tenant_id: i64,
  pub tenant_status: TenantUserStatus,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}
