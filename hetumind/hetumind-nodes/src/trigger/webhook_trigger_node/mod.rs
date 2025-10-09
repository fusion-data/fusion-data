//! # WebhookTriggerNode
//!
//! A node that triggers workflow execution when receiving HTTP webhook requests.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeDefinitionBuilder, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, NodeProperty, RegistrationError,
  make_execution_data_map,
};

use crate::constants::WEBHOOK_TRIGGER_NODE_KIND;

pub struct WebhookTriggerNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinitionBuilder> for WebhookTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(builder: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    let definition = builder.build()?;
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

fn create_base() -> NodeDefinitionBuilder {
  let mut base = NodeDefinitionBuilder::default();
  base
    .kind(WEBHOOK_TRIGGER_NODE_KIND)
    .version(Version::new(1, 0, 0))
    .groups([NodeGroupKind::Trigger])
    .display_name("Webhook Trigger")
    .description("Triggers workflow when HTTP request is received")
    .outputs(vec![])
    .properties(vec![
      // HTTP Method
      NodeProperty::builder()
        .display_name("HTTP Method")
        .name("http_method")
        .kind(hetumind_core::workflow::NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "GET",
            "get",
            JsonValue::String("GET".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "POST",
            "post",
            JsonValue::String("POST".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "PUT",
            "put",
            JsonValue::String("PUT".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "DELETE",
            "delete",
            JsonValue::String("DELETE".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "PATCH",
            "patch",
            JsonValue::String("PATCH".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
        ])
        .required(true)
        .description("HTTP method for the webhook endpoint")
        .build(),
      // Path
      NodeProperty::builder()
        .display_name("Path")
        .name("path")
        .kind(hetumind_core::workflow::NodePropertyKind::String)
        .required(true)
        .description("Webhook path (e.g., /webhook/my-workflow)")
        .hint("e.g., /webhook/my-workflow")
        .build(),
      // Authentication
      NodeProperty::builder()
        .display_name("Authentication")
        .name("authentication")
        .kind(hetumind_core::workflow::NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "None",
            "none",
            JsonValue::String("none".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Basic Auth",
            "basic",
            JsonValue::String("basic".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Bearer Token",
            "bearer",
            JsonValue::String("bearer".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Header Auth",
            "header",
            JsonValue::String("header".to_string()),
            hetumind_core::workflow::NodePropertyKind::String,
          )),
        ])
        .required(true)
        .description("Authentication method for the webhook")
        .build(),
      // Response Code
      NodeProperty::builder()
        .display_name("Response Code")
        .name("response_code")
        .kind(hetumind_core::workflow::NodePropertyKind::Number)
        .required(false)
        .description("HTTP response code to return")
        .value(JsonValue::Number(serde_json::Number::from(200)))
        .build(),
      // Response Body
      NodeProperty::builder()
        .display_name("Response Body")
        .name("response_body")
        .kind(hetumind_core::workflow::NodePropertyKind::String)
        .required(false)
        .description("Response body template (JSON)")
        .value(JsonValue::String("{\"status\": \"success\"}".to_string()))
        .build(),
      // Headers
      NodeProperty::builder()
        .display_name("Response Headers")
        .name("response_headers")
        .kind(hetumind_core::workflow::NodePropertyKind::FixedCollection)
        .required(false)
        .description("Additional response headers")
        .build(),
    ]);
  base
}
