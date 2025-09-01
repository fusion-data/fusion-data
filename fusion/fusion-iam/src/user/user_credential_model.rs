use modelsql::{
  field::Fields,
  filter::{FilterNodes, OpValsInt64, OpValsValue},
  postgres::PgRowType,
  utils::datetime_to_sea_value,
};
use sqlx::FromRow;
use fusion_common::time::UtcDateTime;

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

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub ctime: Option<OpValsValue>,

  pub mid: Option<OpValsInt64>,

  #[modelsql(to_sea_value_fn = "datetime_to_sea_value")]
  pub mtime: Option<OpValsValue>,
}
