use fusion_common::time::OffsetDateTime;
use fusionsql::field::Fields;
use hetumind_core::workflow::CredentialKind;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 凭证实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "credential_entity")]
pub struct CredentialEntity {
  pub id: Uuid,
  pub namespace_id: String,
  pub name: String,
  /// encrypted credential data with jwe
  pub data: String,
  pub kind: CredentialKind,
  pub is_managed: bool,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub created_by: i64,
  pub updated_by: Option<i64>,
  pub deleted_at: Option<OffsetDateTime>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_credentials_models() {
    assert_eq!(CredentialEntityIden::Table.as_ref(), "credential_entity");
  }
}
