use fusion_common::time::UtcDateTime;
use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsDateTime, OpValsInt64},
  postgres::PgRowType,
};
use sqlx::FromRow;

#[derive(FromRow, Fields)]
pub struct UserCredential {
  pub id: i64,
  pub encrypted_pwd: String,
  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl PgRowType for UserCredential {}

#[derive(Fields)]
pub struct UserCredentialForCreate {
  pub id: i64,
  pub encrypted_pwd: String,
}

#[derive(Default, Fields)]
pub struct UserCredentialForUpdate {
  pub id: Option<i64>,
  pub encrypted_pwd: Option<String>,
}

#[derive(Default, FilterNodes)]
pub struct UserCredentialFilter {
  pub id: Option<OpValsInt64>,

  pub cid: Option<OpValsInt64>,

  pub ctime: Option<OpValsDateTime>,

  pub mid: Option<OpValsInt64>,

  pub mtime: Option<OpValsDateTime>,
}
