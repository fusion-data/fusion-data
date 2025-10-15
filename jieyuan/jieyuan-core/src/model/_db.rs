use fusionsql::postgres::PgRowType;
use sea_query::{Nullable, Value};

use super::{Gender, Permission, PolicyEntity, RolePermission, RoleStatus, User, UserCredential, UserRole, UserStatus};

impl PgRowType for PolicyEntity {}
impl PgRowType for User {}
impl PgRowType for UserCredential {}
impl PgRowType for UserRole {}
impl PgRowType for super::Role {}
impl PgRowType for RolePermission {}
impl PgRowType for Permission {}

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

impl From<RoleStatus> for Value {
  fn from(value: RoleStatus) -> Self {
    Value::Int(Some(value as i32))
  }
}
impl Nullable for RoleStatus {
  fn null() -> Value {
    Value::Int(None)
  }
}
