use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 工作流实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflow_entity")]
pub struct WorkflowEntity {
  pub id: String,
  pub name: String,
  pub active: bool,
  pub nodes: serde_json::Value,
  pub connections: serde_json::Value,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub settings: Option<serde_json::Value>,
  pub static_data: Option<serde_json::Value>,
  pub pin_data: Option<serde_json::Value>,
  pub version_id: Option<String>,
  pub trigger_count: i32,
  pub meta: Option<serde_json::Value>,
  pub parent_folder_id: Option<String>,
  pub is_archived: bool,
}

/// 执行实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_entity")]
pub struct ExecutionEntity {
  pub id: i32,
  pub workflow_id: String,
  pub finished: bool,
  pub mode: String,
  pub retry_of: Option<String>,
  pub retry_success_id: Option<String>,
  pub started_at: Option<UtcDateTime>,
  pub stopped_at: Option<UtcDateTime>,
  pub wait_till: Option<UtcDateTime>,
  pub status: String,
  pub deleted_at: Option<UtcDateTime>,
  pub ctime: UtcDateTime,
}

/// 执行数据表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_data")]
pub struct ExecutionData {
  pub execution_id: i32,
  pub workflow_data: serde_json::Value,
  pub data: String,
}

/// 执行注解表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_annotations")]
pub struct ExecutionAnnotations {
  pub id: i32,
  pub execution_id: i32,
  pub vote: Option<String>,
  pub note: Option<String>,
  pub ctime: UtcDateTime,
  pub mtime: UtcDateTime,
}

/// 执行注解标签关联表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_annotation_tags")]
pub struct ExecutionAnnotationTags {
  pub annotation_id: i32,
  pub tag_id: String,
}

/// 执行元数据表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_metadata")]
pub struct ExecutionMetadata {
  pub id: i32,
  pub execution_id: i32,
  pub key: String,
  pub value: String,
}

/// 共享工作流表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "shared_workflow")]
pub struct SharedWorkflow {
  pub workflow_id: String,
  pub project_id: String,
  pub role: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
}

/// 工作流历史表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflow_history")]
pub struct WorkflowHistory {
  pub version_id: String,
  pub workflow_id: String,
  pub authors: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub nodes: serde_json::Value,
  pub connections: serde_json::Value,
}

/// 工作流统计表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "workflow_statistics")]
pub struct WorkflowStatistics {
  pub count: Option<i32>,
  pub latest_event: Option<UtcDateTime>,
  pub name: String,
  pub workflow_id: String,
  pub root_count: Option<i32>,
}

/// Webhook实体表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "webhook_entity")]
pub struct WebhookEntity {
  pub webhook_path: String,
  pub method: String,
  pub node: String,
  pub webhook_id: Option<String>,
  pub path_length: Option<i32>,
  pub workflow_id: String,
}

/// 处理数据表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "processed_data")]
pub struct ProcessedData {
  pub workflow_id: String,
  pub context: String,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub value: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_workflow_models() {
    assert_eq!(WorkflowEntityIden::Table.as_ref(), "workflow_entity");
    assert_eq!(ExecutionEntityIden::Table.as_ref(), "execution_entity");
    assert_eq!(ExecutionDataIden::Table.as_ref(), "execution_data");
    assert_eq!(ExecutionAnnotationsIden::Table.as_ref(), "execution_annotations");
    assert_eq!(ExecutionAnnotationTagsIden::Table.as_ref(), "execution_annotation_tags");
    assert_eq!(ExecutionMetadataIden::Table.as_ref(), "execution_metadata");
    assert_eq!(SharedWorkflowIden::Table.as_ref(), "shared_workflow");
    assert_eq!(WorkflowHistoryIden::Table.as_ref(), "workflow_history");
    assert_eq!(WorkflowStatisticsIden::Table.as_ref(), "workflow_statistics");
    assert_eq!(WebhookEntityIden::Table.as_ref(), "webhook_entity");
    assert_eq!(ProcessedDataIden::Table.as_ref(), "processed_data");
  }
}
