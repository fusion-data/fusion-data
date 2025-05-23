use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt32, OpValsInt64, OpValsString, OpValsValue},
  page::PageResult,
  postgres::PgRowType,
  utils::datetime_to_sea_value,
};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use ultimate_api::v1::{Page, Pagination};
use ultimate_common::{regex, time::UtcDateTime};
use ultimate_core::{DataError, Result};
use ultimate_db::{
  try_into_op_vals_int32_opt, try_into_op_vals_int64_opt, try_into_op_vals_string_opt,
  try_into_op_values_with_string_opt,
};

use crate::pb::fusion_iam::v1::{
  CreateUserRequest, FilterUserRequest, Gender, PageUserRequest, PageUserResponse, UpdateUserRequest, UserDto,
  UserStatus,
};

#[derive(Debug, Serialize, FromRow, Fields)]
#[enum_def]
pub struct User {
  pub id: i64,
  pub email: Option<String>,
  pub phone: Option<String>,
  pub name: String,
  pub status: UserStatus,
  pub gender: Gender,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl PgRowType for User {}

impl From<User> for UserDto {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      email: user.email,
      phone: user.phone,
      name: user.name,
      status: user.status as i32,
      gender: user.gender as i32,
      cid: user.cid,
      ctime: user.ctime.timestamp(),
      mid: user.mid,
      mtime: user.mtime.map(|t| t.timestamp()),
    }
  }
}

#[derive(Debug, Deserialize, Fields)]
pub struct UserForCreate {
  pub email: Option<String>,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub status: Option<UserStatus>,
}

impl UserForCreate {
  /// 校验数据并进行初始化。`email` 或 `phone` 至少有一个，若两个值都设置，则只有 `email` 有效。
  ///
  /// 当 `name` 未设置时，将从 `email` 或 `phone` 中取值。
  pub fn validate_and_init(mut self) -> Result<Self> {
    if let Some(email) = self.email.as_deref() {
      if !regex::is_email(email) {
        return Err(DataError::bad_request("The 'email' field is invalid"));
      }
    } else if let Some(phone) = self.phone.as_deref() {
      if !regex::is_phone(phone) {
        return Err(DataError::bad_request("The 'phone' field is invalid"));
      }
    } else {
      return Err(DataError::bad_request("At least one 'email' or 'phone' is required"));
    };

    let has_name = self.name.as_deref().is_some_and(|n| !n.is_empty());
    if !has_name {
      self.name = match self.email.as_deref() {
        Some(email) => email.split('@').next().map(ToString::to_string),
        None => self.phone.clone(),
      };
    }

    Ok(self)
  }
}

#[derive(Debug, Deserialize, Fields)]
pub struct UserForUpdate {
  pub name: Option<String>,
  pub status: Option<UserStatus>,
}

impl TryFrom<UpdateUserRequest> for UserForUpdate {
  type Error = DataError;
  fn try_from(value: UpdateUserRequest) -> core::result::Result<Self, DataError> {
    let status = match value.status {
      Some(i) => Some(UserStatus::try_from(i)?),
      None => None,
    };
    Ok(Self { name: value.name, status })
  }
}

#[derive(Debug, Default, Deserialize)]
pub struct UserForPage {
  pub page: Pagination,
  pub filter: Vec<UserFilter>,
}

#[derive(Debug, Default, Deserialize, FilterNodes)]
pub struct UserFilter {
  pub id: Option<OpValsInt64>,

  pub email: Option<OpValsString>,

  pub phone: Option<OpValsString>,

  pub name: Option<OpValsString>,

  pub status: Option<OpValsInt32>,

  pub gender: Option<OpValsInt32>,

  pub cid: Option<OpValsInt64>,

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  pub mid: Option<OpValsInt64>,

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}

#[derive(Debug, Serialize)]
pub struct UserPage {
  pub page: modelsql::page::Page,
  pub items: Vec<User>,
}

impl From<PageResult<User>> for UserPage {
  fn from(value: PageResult<User>) -> Self {
    Self { page: value.page, items: value.result }
  }
}

impl From<UserStatus> for sea_query::Value {
  fn from(value: UserStatus) -> Self {
    sea_query::Value::Int(Some(value as i32))
  }
}

impl sea_query::Nullable for UserStatus {
  fn null() -> sea_query::Value {
    sea_query::Value::Int(None)
  }
}

impl From<Gender> for sea_query::Value {
  fn from(value: Gender) -> Self {
    sea_query::Value::Int(Some(value as i32))
  }
}

impl sea_query::Nullable for Gender {
  fn null() -> sea_query::Value {
    sea_query::Value::Int(None)
  }
}

impl TryFrom<CreateUserRequest> for UserForCreate {
  type Error = DataError;
  fn try_from(value: CreateUserRequest) -> core::result::Result<Self, DataError> {
    let status = match value.status {
      Some(i) => Some(UserStatus::try_from(i)?),
      None => None,
    };
    Ok(Self { email: value.email, phone: value.phone, name: value.name, status })
  }
}

impl From<PageUserRequest> for UserForPage {
  fn from(value: PageUserRequest) -> Self {
    let filter = value.filter.into_iter().map(UserFilter::from).collect();
    let page = value.pagination.unwrap_or_default();
    Self { page, filter }
  }
}

impl From<FilterUserRequest> for UserFilter {
  fn from(value: FilterUserRequest) -> Self {
    Self {
      id: try_into_op_vals_int64_opt(value.id).unwrap(),
      email: try_into_op_vals_string_opt(value.email).unwrap(),
      phone: try_into_op_vals_string_opt(value.phone).unwrap(),
      name: try_into_op_vals_string_opt(value.name).unwrap(),
      status: try_into_op_vals_int32_opt(value.status).unwrap(),
      gender: try_into_op_vals_int32_opt(value.gender).unwrap(),
      cid: try_into_op_vals_int64_opt(value.cid).unwrap(),
      ctime: try_into_op_values_with_string_opt(value.ctime).unwrap(),
      mid: try_into_op_vals_int64_opt(value.mid).unwrap(),
      mtime: try_into_op_values_with_string_opt(value.mtime).unwrap(),
    }
  }
}

impl From<UserPage> for PageUserResponse {
  fn from(value: UserPage) -> Self {
    let items = value.items.into_iter().map(UserDto::from).collect();
    Self { page: Some(Page::new(value.page.total)), items }
  }
}
