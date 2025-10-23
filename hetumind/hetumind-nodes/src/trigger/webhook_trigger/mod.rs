//! # WebhookTriggerNode
//!
//! A node that triggers workflow execution when receiving HTTP webhook requests.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeExecutable, NodeExecutionContext,
  NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, NodeProperty, NodePropertyKind, RegistrationError,
  make_execution_data_map,
};

use crate::constants::WEBHOOK_TRIGGER_NODE_KIND;
use serde_json::json;

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
  NodeDefinition::new(WEBHOOK_TRIGGER_NODE_KIND, "Webhook Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("Triggers workflow when HTTP request is received")
    .add_property(
      // HTTP Method
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("HTTP Method")
        .with_name("http_method")
        .with_options(vec![
          Box::new(NodeProperty::new_option("GET", "get", json!("GET"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("POST", "post", json!("POST"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("PUT", "put", json!("PUT"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("DELETE", "delete", json!("DELETE"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("PATCH", "patch", json!("PATCH"), NodePropertyKind::String)),
        ])
        .with_required(true)
        .with_description("HTTP method for the webhook endpoint"),
    )
    .add_property(
      // Path
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Path")
        .with_name("path")
        .with_required(true)
        .with_description("Webhook path (e.g., /webhook/my-workflow)")
        .with_hint("e.g., /webhook/my-workflow"),
    )
    .add_property(
      // Authentication
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Authentication")
        .with_name("authentication")
        .with_options(vec![
          Box::new(NodeProperty::new_option("None", "none", json!("none"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Basic Auth", "basic", json!("basic"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Bearer Token", "bearer", json!("bearer"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("Header Auth", "header", json!("header"), NodePropertyKind::String)),
        ])
        .with_required(true)
        .with_description("Authentication method for the webhook"),
    )
    .add_property(
      // Response Code
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Response Code")
        .with_name("response_code")
        .with_required(false)
        .with_description("HTTP response code to return")
        .with_value(json!(200)),
    )
    .add_property(
      // Response Body
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Response Body")
        .with_name("response_body")
        .with_required(false)
        .with_description("Response body template (JSON)")
        .with_value(json!("{\"status\": \"success\"}")),
    )
    .add_property(
      // Headers
      NodeProperty::new(NodePropertyKind::FixedCollection)
        .with_display_name("Response Headers")
        .with_name("response_headers")
        .with_required(false)
        .with_description("Additional response headers"),
    )
}
