use fusion_common::{regex, time::OffsetDateTime};
use fusion_core::{DataError, Result};
use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsDateTime, OpValsInt32, OpValsInt64, OpValsString, Page},
  postgres::PgRowType,
};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::pb::{Gender, UserDto, UserStatus};

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
  pub ctime: OffsetDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<OffsetDateTime>,
}
impl PgRowType for User {}

impl From<User> for UserDto {
  fn from(user: User) -> Self {
    Self {
      id: user.id,
      email: user.email,
      phone: user.phone,
      name: user.name,
      status: user.status,
      gender: user.gender,
      cid: user.cid,
      ctime: user.ctime,
      mid: user.mid,
      mtime: user.mtime,
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

#[derive(Debug, Default, Deserialize)]
pub struct UserForPage {
  pub page: Page,
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

  pub ctime: Option<OpValsDateTime>,

  pub mid: Option<OpValsInt64>,

  pub mtime: Option<OpValsDateTime>,
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
