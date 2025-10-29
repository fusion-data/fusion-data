use chrono::{DateTime, FixedOffset};
use fusion_common::model::sensitive::SensitiveString;
use fusion_common::page::Page;
use fusionsql::generate_enum_i32_to_sea_query_value;
use fusionsql::{
  field::{FieldMask, Fields},
  filter::{FilterNodes, OpValDateTime, OpValInt32, OpValString, OpValUuid},
  postgres::PgRowType,
};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, sqlx::Type)]
#[repr(i32)]
pub enum UserStatus {
  // new user status
  // NotActive = 1,
  Disabled = 99,
  Enabled = 100,
}
generate_enum_i32_to_sea_query_value!(
  Enum: UserStatus,
);

#[derive(Debug, Serialize, FromRow, Fields)]
#[enum_def(table_name = "user_entity")]
pub struct UserEntity {
  pub id: i64,
  pub tenant_id: i64,
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  #[serde(skip_serializing)]
  pub password: Option<SensitiveString>,
  pub personalization_answers: Option<serde_json::Value>,
  pub settings: Option<serde_json::Value>,
  pub status: UserStatus,
  pub mfa_enabled: bool,
  pub mfa_secret: Option<String>,
  pub mfa_recovery_codes: Option<String>,
  // pub role: String,
  pub created_at: DateTime<FixedOffset>,
  pub created_by: i64,
  pub updated_at: Option<DateTime<FixedOffset>>,
  pub updated_by: Option<i64>,
}
impl PgRowType for UserEntity {}

#[derive(Debug, Deserialize, Fields)]
pub struct UserForCreate {
  pub tenant_id: i64,
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub password: String,
  pub status: UserStatus,
}

#[derive(Clone, Default, Deserialize, Fields)]
pub struct UserForUpdate {
  pub email: Option<String>,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub status: Option<UserStatus>,
  pub update_mask: Option<FieldMask>,
}

/// 更新密码。
/// 1. 如果 old_password 不为空，则需要验证 old_password 是否正确。
/// 2. 如果 code 不为空，则需要验证 code 是否正确。
/// 3. old_password 和 code 都为空，则需要验证当前用户是否是管理人。
#[derive(Debug, Deserialize, Fields)]
pub struct UserForUpdatePassword {
  #[fusionsql(skip)]
  pub old_password: Option<String>,
  #[fusionsql(skip)]
  pub code: Option<String>,
  pub password: String,
}

#[derive(Default, Deserialize, FilterNodes)]
pub struct UserFilter {
  pub id: Option<OpValUuid>,
  pub email: Option<OpValString>,
  pub phone: Option<OpValString>,
  pub name: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub created_at: Option<OpValDateTime>,
  pub updated_at: Option<OpValDateTime>,
}

#[derive(Deserialize)]
pub struct UserForPage {
  pub page: Page,
  pub filter: UserFilter,
}
