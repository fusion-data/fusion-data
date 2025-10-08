use std::fmt::{self, Debug};

use fusion_common::page::Page;
use fusion_common::time::OffsetDateTime;
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

#[derive(Serialize, FromRow, Fields)]
#[enum_def(table_name = "user_entity")]
pub struct UserEntity {
  pub id: i64,
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  #[serde(skip_serializing)]
  pub password: Option<String>,
  pub personalization_answers: Option<serde_json::Value>,
  pub settings: Option<serde_json::Value>,
  pub status: UserStatus,
  pub mfa_enabled: bool,
  pub mfa_secret: Option<String>,
  pub mfa_recovery_codes: Option<String>,
  // pub role: String,
  pub created_at: OffsetDateTime,
  pub created_by: i64,
  pub updated_at: Option<OffsetDateTime>,
  pub updated_by: Option<i64>,
}
impl PgRowType for UserEntity {}

impl Debug for UserEntity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "UserEntity {{ id: {}, email: {}, phone: {:?}, name: {:?}, password: <hidden>, personalization_answers: {:?}, settings: {:?}, status: {:?}, mfa_enabled: {:?}, mfa_secret: {:?}, mfa_recovery_codes: {:?}, created_at: {:?}, created_by: {}, updated_at: {:?}, updated_by: {:?} }}",
      self.id,
      self.email,
      self.phone,
      self.name,
      self.personalization_answers,
      self.settings,
      self.status,
      self.mfa_enabled,
      self.mfa_secret,
      self.mfa_recovery_codes,
      self.created_at,
      self.created_by,
      self.updated_at,
      self.updated_by
    )
  }
}

#[derive(Debug, Deserialize, Fields)]
pub struct UserForCreate {
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub password: String,
  pub status: UserStatus,
}

#[derive(Clone, Deserialize, Fields)]
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

#[cfg(test)]
mod tests {
  use fusion_common::time::now;
  use fusionsql::field::HasSeaFields;
  use sea_query::ColumnRef;

  use super::*;

  #[test]
  fn test_user_entity_display() {
    let user = UserEntity {
      id: 1,
      email: "test@test.com".to_string(),
      phone: Some("12345678901".to_string()),
      name: Some("test".to_string()),
      password: Some("123456".to_string()),
      personalization_answers: Some(serde_json::json!({})),
      settings: Some(serde_json::json!({})),
      status: UserStatus::Enabled,
      mfa_enabled: false,
      mfa_secret: None,
      mfa_recovery_codes: None,
      created_at: now(),
      created_by: 1,
      updated_at: None,
      updated_by: None,
    };
    println!("{:?}", user);
  }

  #[test]
  fn test_user_for_update() {
    let mut for_update = UserForUpdate {
      email: Some("test@test.com".to_string()),
      phone: Some("12345678901".to_string()),
      name: None,
      status: Some(UserStatus::Enabled),
      update_mask: Default::default(),
    };
    let non_empty_fields: Vec<String> = for_update
      .clone()
      .not_none_sea_fields()
      .into_iter()
      .map(|f| match f.column_ref {
        ColumnRef::Column(iden) => iden.to_string(),
        _ => panic!("unexpected column ref"),
      })
      .collect::<Vec<_>>();
    assert_eq!(non_empty_fields, vec!["email", "phone", "status"]);

    let sea_fields_with_mask: Vec<String> = for_update
      .clone()
      .sea_fields_with_mask()
      .into_iter()
      .map(|f| match f.column_ref {
        ColumnRef::Column(iden) => iden.to_string(),
        _ => panic!("unexpected column ref"),
      })
      .collect::<Vec<_>>();
    assert_eq!(sea_fields_with_mask, vec!["email", "phone", "status"]);

    for_update.update_mask = Some(FieldMask::new(vec!["phone".to_string(), "name".to_string()]));
    let sea_fields_with_mask: Vec<String> = for_update
      .sea_fields_with_mask()
      .into_iter()
      .map(|f| match f.column_ref {
        ColumnRef::Column(iden) => iden.to_string(),
        _ => panic!("unexpected column ref"),
      })
      .collect::<Vec<_>>();
    assert_eq!(sea_fields_with_mask, vec!["phone", "name"]);
  }
}
