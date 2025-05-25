use modelsql::field::Fields;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::UtcDateTime;

/// 测试定义表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "test_definition")]
pub struct TestDefinition {
  pub id: String,
  pub name: String,
  pub workflow_id: String,
  pub evaluation_workflow_id: Option<String>,
  pub annotation_tag_id: Option<String>,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
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
  pub ctime: UtcDateTime,
  pub mtime: UtcDateTime,
}

/// 测试运行表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "test_run")]
pub struct TestRun {
  pub id: String,
  pub test_definition_id: String,
  pub status: String,
  pub run_at: Option<UtcDateTime>,
  pub completed_at: Option<UtcDateTime>,
  pub metrics: Option<serde_json::Value>,
  pub ctime: UtcDateTime,
  pub mtime: Option<UtcDateTime>,
  pub total_cases: Option<i32>,
  pub passed_cases: Option<i32>,
  pub failed_cases: Option<i32>,
  pub error_code: Option<String>,
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
  pub run_at: Option<UtcDateTime>,
  pub completed_at: Option<UtcDateTime>,
  pub error_code: Option<String>,
  pub error_details: Option<serde_json::Value>,
  pub metrics: Option<serde_json::Value>,
  pub ctime: UtcDateTime,
  pub mtime: UtcDateTime,
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
