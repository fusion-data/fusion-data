use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{NodeDefinitionBuilder, NodeExecutionContext, NodeGroupKind, NodeProperty, NodePropertyKind},
};
use serde_json::json;

use crate::constants::ERROR_TRIGGER_NODE_KIND;

pub fn create_base() -> NodeDefinitionBuilder {
  let mut base = NodeDefinitionBuilder::default();
  base
    .kind(ERROR_TRIGGER_NODE_KIND)
    .version(Version::new(1, 0, 0))
    .groups([NodeGroupKind::Trigger])
    .display_name("Error Trigger")
    .description("Triggers workflow when other workflows encounter errors")
    .max_nodes(1)
    .outputs(vec![])
    .properties(vec![
      // 触发模式
      NodeProperty::builder()
        .display_name("Trigger Mode")
        .name("trigger_mode")
        .kind(NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "All Workflows",
            "all_workflows",
            JsonValue::String("all_workflows".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Specific Workflows",
            "specific_workflows",
            JsonValue::String("specific_workflows".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Internal Only",
            "internal_only",
            JsonValue::String("internal_only".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(true)
        .description("Select which workflows to monitor for errors")
        .build(),
      // 监听工作流ID
      NodeProperty::builder()
        .display_name("Workflow IDs")
        .name("workflow_ids")
        .kind(NodePropertyKind::MultiOptions)
        .required(false)
        .description("Specific workflow IDs to monitor (comma-separated)")
        .hint("Leave empty to monitor all workflows")
        .build(),
      // 错误类型过滤
      NodeProperty::builder()
        .display_name("Error Types")
        .name("error_types")
        .kind(NodePropertyKind::MultiOptions)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "Node Execution",
            "node_execution",
            JsonValue::String("node_execution".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Timeout",
            "timeout",
            JsonValue::String("timeout".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Resource Exhausted",
            "resource_exhausted",
            JsonValue::String("resource_exhausted".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "External Service",
            "external_service",
            JsonValue::String("external_service".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Validation",
            "validation",
            JsonValue::String("validation".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Configuration",
            "configuration",
            JsonValue::String("configuration".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(false)
        .description("Types of errors to trigger on")
        .build(),
      // 节点名称过滤
      NodeProperty::builder()
        .display_name("Node Names")
        .name("node_names")
        .kind(NodePropertyKind::MultiOptions)
        .required(false)
        .description("Specific node names to monitor (comma-separated)")
        .hint("Leave empty to monitor all nodes")
        .build(),
      // 错误严重级别
      NodeProperty::builder()
        .display_name("Error Severity")
        .name("error_severity")
        .kind(NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "Low",
            "low",
            JsonValue::String("low".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Medium",
            "medium",
            JsonValue::String("medium".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "High",
            "high",
            JsonValue::String("high".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Critical",
            "critical",
            JsonValue::String("critical".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(false)
        .description("Minimum error severity to trigger")
        .build(),
      // 重试配置
      NodeProperty::builder()
        .display_name("Enable Retry")
        .name("enable_retry")
        .kind(NodePropertyKind::Boolean)
        .required(false)
        .description("Enable automatic retry for failed executions")
        .value(JsonValue::Bool(false))
        .build(),
      // 最大重试次数
      NodeProperty::builder()
        .display_name("Max Retry Count")
        .name("max_retry_count")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Maximum number of retry attempts")
        .value(JsonValue::Number(serde_json::Number::from(3)))
        .build(),
      // 重试间隔
      NodeProperty::builder()
        .display_name("Retry Interval (seconds)")
        .name("retry_interval_seconds")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Interval between retry attempts in seconds")
        .value(JsonValue::Number(serde_json::Number::from(60)))
        .build(),
      // 发送通知
      NodeProperty::builder()
        .display_name("Send Notification")
        .name("send_notification")
        .kind(NodePropertyKind::Boolean)
        .required(false)
        .description("Send notifications when errors occur")
        .value(JsonValue::Bool(true))
        .build(),
      // 通知方式
      NodeProperty::builder()
        .display_name("Notification Methods")
        .name("notification_methods")
        .kind(NodePropertyKind::MultiOptions)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "Email",
            "email",
            JsonValue::String("email".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Slack",
            "slack",
            JsonValue::String("slack".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Webhook",
            "webhook",
            JsonValue::String("webhook".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Database",
            "database",
            JsonValue::String("database".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(false)
        .description("Methods to send notifications")
        .build(),
    ]);
  base
}

/// 检查是否为手动测试模式
pub fn is_manual_test_mode(context: &NodeExecutionContext) -> bool {
  // 检查执行环境是否为手动测试模式
  context.input_data.is_empty() && context.user_id.is_some() && context.env_vars.contains_key("MANUAL_TEST")
}

/// 生成示例错误数据
pub fn generate_sample_error_data() -> JsonValue {
  json!({
    "workflow": {
      "id": "example-workflow-123",
      "name": "Example Workflow"
    },
    "execution": {
      "id": "execution-456",
      "url": "/workflow/execution/456",
      "retry_of": null,
      "error": {
        "message": "Example error message for testing",
        "stack": "at Node.execute (/path/to/node.js:42:15)\n  at Workflow.run (/path/to/workflow.js:123:10)",
        "name": "NodeExecutionError",
        "description": "This is a sample error for manual testing",
        "timestamp": "2024-01-15T10:30:00Z"
      },
      "last_node_executed": "failing-node",
      "mode": "manual"
    }
  })
}
