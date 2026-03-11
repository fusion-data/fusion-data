use hetus::common::time::OffsetDateTime;

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, hetusql::Fields), sea_query::enum_def)]
pub struct UserRole {
  pub user_id: i64,
  pub role_id: i64,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(hetusql::Fields))]
pub struct UserRoleForCreate {
  pub user_id: i64,
  pub role_id: i64,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(hetusql::Fields))]
pub struct UserRoleForUpdate {
  pub user_id: Option<i64>,
  pub role_id: Option<i64>,
}
