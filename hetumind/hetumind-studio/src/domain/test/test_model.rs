use fusion_common::time::OffsetDateTime;
use fusionsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 测试定义表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "test_definition")]
pub struct TestDefinition {
  pub id: String,
  pub name: String,
  pub workflow_id: String,
  pub evaluation_workflow_id: Option<String>,
  pub annotation_tag_id: Option<String>,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub description: Option<String>,
  pub mocked_nodes: serde_json::Value,
}

/// 测试指标表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "test_metric")]
pub struct TestMetric {
  pub id: String,
  pub name: String,
  pub test_definition_id: String,
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}

/// 测试运行表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "test_run")]
pub struct TestRun {
  pub id: String,
  pub test_definition_id: String,
  pub status: String,
  pub run_at: Option<OffsetDateTime>,
  pub completed_at: Option<OffsetDateTime>,
  pub metrics: Option<serde_json::Value>,
  pub created_at: OffsetDateTime,
  pub updated_at: Option<OffsetDateTime>,
  pub total_cases: Option<i32>,
  pub passed_cases: Option<i32>,
  pub failed_cases: Option<i32>,
  pub err_code: Option<String>,
  pub error_details: Option<String>,
}

/// 测试用例执行表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "test_case_execution")]
pub struct TestCaseExecution {
  pub id: String,
  pub test_run_id: String,
  pub past_execution_id: Option<i32>,
  pub execution_id: Option<i32>,
  pub evaluation_execution_id: Option<i32>,
  pub status: String,
  pub run_at: Option<OffsetDateTime>,
  pub completed_at: Option<OffsetDateTime>,
  pub err_code: Option<String>,
  pub error_details: Option<serde_json::Value>,
  pub metrics: Option<serde_json::Value>,
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_test_models() {
    assert_eq!(TestDefinitionIden::Table.as_ref(), "test_definition");
    assert_eq!(TestMetricIden::Table.as_ref(), "test_metric");
    assert_eq!(TestRunIden::Table.as_ref(), "test_run");
    assert_eq!(TestCaseExecutionIden::Table.as_ref(), "test_case_execution");
  }
}
