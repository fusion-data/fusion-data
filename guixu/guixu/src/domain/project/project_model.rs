use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;
use uuid::Uuid;

/// 项目表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "project")]
pub struct Project {
  pub id: String,
  pub name: String,
  pub kind: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub icon: Option<serde_json::Value>,
}

/// 文件夹表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "folder")]
pub struct Folder {
  pub id: String,
  pub name: String,
  pub parent_folder_id: Option<String>,
  pub project_id: String,
  pub ctime: UtcDateTime,
  pub mtime: UtcDateTime,
}

/// 项目关系表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "project_relation")]
pub struct ProjectRelation {
  pub project_id: String,
  pub user_id: Uuid,
  pub role: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
}

/// 共享凭证表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "shared_credentials")]
pub struct SharedCredentials {
  pub credentials_id: String,
  pub project_id: String,
  pub role: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_project_models() {
    assert_eq!(ProjectIden::Table.as_ref(), "project");
    assert_eq!(FolderIden::Table.as_ref(), "folder");
    assert_eq!(ProjectRelationIden::Table.as_ref(), "project_relation");
    assert_eq!(SharedCredentialsIden::Table.as_ref(), "shared_credentials");
  }
}
