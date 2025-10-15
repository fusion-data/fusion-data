use hetumind_core::{
  types::JsonValue,
  workflow::{NodeDefinition, NodeExecutionContext, NodeGroupKind, NodeProperty, NodePropertyKind},
};
use serde_json::json;

use crate::constants::ERROR_TRIGGER_NODE_KIND;

pub fn create_base() -> NodeDefinition {
  NodeDefinition::new(ERROR_TRIGGER_NODE_KIND, "Error Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("Triggers workflow when other workflows encounter errors")
    .with_max_nodes(1)
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Trigger Mode")
        .with_name("trigger_mode")
        .with_options(vec![
          Box::new(NodeProperty::new_option(
            "All Workflows",
            "all_workflows",
            json!("all_workflows"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Specific Workflows",
            "specific_workflows",
            json!("specific_workflows"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Internal Only",
            "internal_only",
            json!("internal_only"),
            NodePropertyKind::String,
          )),
        ])
        .with_required(true)
        .with_description("Select which workflows to monitor for errors"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::MultiOptions)
        .with_display_name("Workflow IDs")
        .with_name("workflow_ids")
        .with_required(false)
        .with_description("Specific workflow IDs to monitor (comma-separated)")
        .with_hint("Leave empty to monitor all workflows"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::MultiOptions)
        .with_display_name("Error Types")
        .with_name("error_types")
        .with_options(vec![
          Box::new(NodeProperty::new_option(
            "Node Execution",
            "node_execution",
            json!("node_execution"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Timeout",
            "timeout",
            json!("timeout"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Resource Exhausted",
            "resource_exhausted",
            json!("resource_exhausted"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "External Service",
            "external_service",
            json!("external_service"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Validation",
            "validation",
            json!("validation"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Configuration",
            "configuration",
            json!("configuration"),
            NodePropertyKind::String,
          )),
        ])
        .with_required(false)
        .with_description("Types of errors to trigger on"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::MultiOptions)
        .with_display_name("Node Names")
        .with_name("node_names")
        .with_required(false)
        .with_description("Specific node names to monitor (comma-separated)")
        .with_hint("Leave empty to monitor all nodes"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Error Severity")
        .with_name("error_severity")
        .with_options(vec![
          Box::new(NodeProperty::new_option(
            "Low",
            "low",
            json!("low"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Medium",
            "medium",
            json!("medium"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "High",
            "high",
            json!("high"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Critical",
            "critical",
            json!("critical"),
            NodePropertyKind::String,
          )),
        ])
        .with_required(false)
        .with_description("Minimum error severity to trigger"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("Enable Retry")
        .with_name("enable_retry")
        .with_required(false)
        .with_description("Enable automatic retry for failed executions")
        .with_value(json!(false)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Max Retry Count")
        .with_name("max_retry_count")
        .with_required(false)
        .with_description("Maximum number of retry attempts")
        .with_value(json!(3)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Retry Interval (seconds)")
        .with_name("retry_interval_seconds")
        .with_required(false)
        .with_description("Interval between retry attempts in seconds")
        .with_value(json!(60)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("Send Notification")
        .with_name("send_notification")
        .with_required(false)
        .with_description("Send notifications when errors occur")
        .with_value(json!(true)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::MultiOptions)
        .with_display_name("Notification Methods")
        .with_name("notification_methods")
        .with_options(vec![
          Box::new(NodeProperty::new_option(
            "Email",
            "email",
            json!("email"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Slack",
            "slack",
            json!("slack"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Webhook",
            "webhook",
            json!("webhook"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Database",
            "database",
            json!("database"),
            NodePropertyKind::String,
          )),
        ])
        .with_required(false)
        .with_description("Methods to send notifications"),
    )
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
