use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::OffsetDateTime;
use uuid::Uuid;

/// 凭证实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "credential_entity")]
pub struct CredentialEntity {
  pub id: Uuid,
  pub name: String,
  pub data: String,
  pub kind: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub is_managed: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_credentials_models() {
    assert_eq!(CredentialEntityIden::Table.as_ref(), "credential_entity");
  }
}
