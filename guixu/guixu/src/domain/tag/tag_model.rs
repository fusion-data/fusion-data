use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 标签实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "tag_entity")]
pub struct TagEntity {
  pub id: String,
  pub name: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
}

/// 注解标签实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "annotation_tag_entity")]
pub struct AnnotationTagEntity {
  pub id: String,
  pub name: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
}

/// 文件夹标签关联表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "folder_tag")]
pub struct FolderTag {
  pub folder_id: String,
  pub tag_id: String,
}

/// 工作流标签关联表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflows_tags")]
pub struct WorkflowsTags {
  pub workflow_id: String,
  pub tag_id: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tag_models() {
    assert_eq!(TagEntityIden::Table.as_ref(), "tag_entity");
    assert_eq!(AnnotationTagEntityIden::Table.as_ref(), "annotation_tag_entity");
    assert_eq!(FolderTagIden::Table.as_ref(), "folder_tag");
    assert_eq!(WorkflowsTagsIden::Table.as_ref(), "workflows_tags");
  }
}
