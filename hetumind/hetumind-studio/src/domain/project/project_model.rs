use fusion_common::time::OffsetDateTime;
use fusionsql::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 项目表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "project")]
pub struct ProjectEntity {
  pub id: Uuid,
  pub name: String,
  pub kind: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub icon: Option<serde_json::Value>,
}

/// 文件夹表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "folder")]
pub struct FolderEntity {
  pub id: Uuid,
  pub name: String,
  pub parent_folder_id: Option<Uuid>,
  pub project_id: Uuid,
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}

/// 项目关系表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "project_relation")]
pub struct ProjectRelationEntity {
  pub project_id: Uuid,
  pub user_id: i64,
  pub role: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
}

/// 共享凭证表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "shared_credentials")]
pub struct SharedCredentials {
  pub credentials_id: Uuid,
  pub project_id: Uuid,
  pub role: String,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_project_models() {
    assert_eq!(ProjectEntityIden::Table.as_ref(), "project");
    assert_eq!(FolderEntityIden::Table.as_ref(), "folder");
    assert_eq!(ProjectRelationEntityIden::Table.as_ref(), "project_relation");
    assert_eq!(SharedCredentialsIden::Table.as_ref(), "shared_credentials");
  }
}
