use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 凭证实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "credentials_entity")]
pub struct CredentialsEntity {
  pub id: String,
  pub name: String,
  pub data: String,
  pub kind: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub is_managed: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_credentials_models() {
    assert_eq!(CredentialsEntityIden::Table.as_ref(), "credentials_entity");
  }
}
