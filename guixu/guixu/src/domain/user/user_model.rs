use modelsql::field::Fields;
use sea_query::enum_def;
use serde::Serialize;
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

#[derive(Debug, Serialize, FromRow, Fields)]
#[enum_def(table_name = "guixu_user")]
pub struct User {
  pub id: i64,
  pub name: String,
  pub email: String,
  pub password: String,
  pub ctime: UtcDateTime,
  pub cid: i64,
  pub utime: Option<UtcDateTime>,
  pub uid: Option<i64>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_user_model() {
    assert_eq!(UserIden::Table.as_ref(), "guixu_user");
  }
}
