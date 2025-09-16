use fusion_common::time::OffsetDateTime;

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, modelsql::field::Fields), sea_query::enum_def)]
pub struct UserRole {
  pub user_id: i64,
  pub role_id: i64,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
pub struct UserRoleForCreate {
  pub user_id: i64,
  pub role_id: i64,
}

#[derive(Debug)]
#[cfg_attr(feature = "with-db", derive(modelsql::field::Fields))]
pub struct UserRoleForUpdate {
  pub user_id: Option<i64>,
  pub role_id: Option<i64>,
}
