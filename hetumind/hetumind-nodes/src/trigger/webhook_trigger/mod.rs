//! # WebhookTriggerNode
//!
//! A node that triggers workflow execution when receiving HTTP webhook requests.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeExecutable, NodeExecutionContext,
  NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, NodeProperty, NodePropertyKind, RegistrationError,
  make_execution_data_map,
};

use crate::constants::WEBHOOK_TRIGGER_NODE_KIND;

pub struct WebhookTriggerNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for WebhookTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for WebhookTriggerNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, _context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // Webhook 触发器作为入口点，返回空数据
    // 实际的 webhook 数据处理在触发器框架层面完成
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

pub struct WebhookTriggerNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl Node for WebhookTriggerNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

impl WebhookTriggerNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(WebhookTriggerNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

fn create_base() -> NodeDefinition {
  NodeDefinition::new(WEBHOOK_TRIGGER_NODE_KIND, Version::new(1, 0, 0), "Webhook Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("Triggers workflow when HTTP request is received")
    .add_property(
      // HTTP Method
      NodeProperty::builder()
        .display_name("HTTP Method")
        .name("http_method")
        .kind(NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "GET",
            "get",
            JsonValue::String("GET".to_string()),
            NodePropertyKind::String,

          )),
          Box::new(NodeProperty::new_option(
            "POST",
            "post",
            JsonValue::String("POST".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "PUT",
            "put",
            JsonValue::String("PUT".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "DELETE",
            "delete",
            JsonValue::String("DELETE".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "PATCH",
            "patch",
            JsonValue::String("PATCH".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(true)
        .description("HTTP method for the webhook endpoint")
        .build(),
    )
    .add_property(
      // Path
      NodeProperty::builder()
        .display_name("Path")
        .name("path")
        .kind(NodePropertyKind::String)
        .required(true)
        .description("Webhook path (e.g., /webhook/my-workflow)")
        .hint("e.g., /webhook/my-workflow")
        .build(),
    )
    .add_property(
      // Authentication
      NodeProperty::builder()
        .display_name("Authentication")
        .name("authentication")
        .kind(NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "None",
            "none",
            JsonValue::String("none".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Basic Auth",
            "basic",
            JsonValue::String("basic".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Bearer Token",
            "bearer",
            JsonValue::String("bearer".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Header Auth",
            "header",
            JsonValue::String("header".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(true)
        .description("Authentication method for the webhook")
        .build(),
    )
    .add_property(
      // Response Code
      NodeProperty::builder()
        .display_name("Response Code")
        .name("response_code")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("HTTP response code to return")
        .value(JsonValue::Number(serde_json::Number::from(200)))
        .build(),
    )
    .add_property(
      // Response Body
      NodeProperty::builder()
        .display_name("Response Body")
        .name("response_body")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Response body template (JSON)")
        .value(JsonValue::String("{\"status\": \"success\"}".to_string()))
        .build(),
    )
    .add_property(
      // Headers
      NodeProperty::builder()
        .display_name("Response Headers")
        .name("response_headers")
        .kind(NodePropertyKind::FixedCollection)
        .required(false)
        .description("Additional response headers")
        .build(),
    )
}
