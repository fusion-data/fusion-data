//! HTTP 请求节点实现

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
  NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeGroupKind, NodeProperty, NodePropertyKind,
  OutputPortConfig, RegistrationError, make_execution_data_map,
};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::{Value, json};

use crate::constants::HTTP_REQUEST_NODE_KIND;

use super::HttpMethod;

/// HTTP Request Node
///
/// Used to send HTTP requests and retrieve response data. Supports common methods such as GET, POST, PUT, DELETE, etc.
///
/// # Parameters
/// - `url`: Target URL for the request
/// - `method`: [HttpMethod] (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
/// - `headers`: Request headers (optional)
/// - `body`: Request body (optional, applicable to POST/PUT/PATCH)
/// - `timeout`: Timeout in seconds (optional, default 30 seconds)
/// - `follow_redirects`: Whether to follow redirects (optional, default true)
/// - `max_redirects`: Maximum number of redirects (optional, default 10)
#[derive(Debug, Clone)]
pub struct HttpRequestV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for HttpRequestV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for HttpRequestV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    info!(
      "Starting HTTP request node workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // Get required parameters
    let url = node.parameters.get_parameter::<String>("url")?;
    let method = node.parameters.get_parameter::<HttpMethod>("method").unwrap_or(HttpMethod::Get);

    // Get optional parameters
    let headers = node
      .parameters
      .get_optional_parameter::<serde_json::Map<String, Value>>("headers")
      .unwrap_or_default();
    let body = node.parameters.get_optional_parameter::<Value>("body");
    let timeout = node.parameters.get_optional_parameter::<u64>("timeout").unwrap_or(30);
    let follow_redirects = node.parameters.get_optional_parameter::<bool>("follow_redirects").unwrap_or(true);

    debug!(
      "HTTP 请求参数: method={:?}, url={}, timeout={}s, follow_redirects={}",
      method, url, timeout, follow_redirects
    );

    // 创建 HTTP 客户端
    let client = Client::builder()
      .user_agent("Hetumind/1.0")
      .timeout(Duration::from_secs(timeout))
      .redirect(if follow_redirects {
        reqwest::redirect::Policy::limited(10)
      } else {
        reqwest::redirect::Policy::none()
      })
      .build()
      .map_err(|e| NodeExecutionError::InitFailed { message: "HTTP Client".to_string(), cause: Some(Box::new(e)) })?;

    let mut request_builder = method.create_request_builder(&client, &url);

    // Add http request headers
    for (key, value) in headers {
      if let Some(header_value) = value.as_str() {
        request_builder = request_builder.header(&key, header_value);
      }
    }

    // Add http request body (if exists and method supports)
    if let Some(body_data) = body {
      if matches!(method, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch) {
        request_builder = request_builder.json(&body_data);
      } else {
        warn!("HTTP method {:?} does not support request body, ignore body parameter", method);
      }
    }

    // Send http request
    let start_time = std::time::Instant::now();
    let response = request_builder.send().await.map_err(|e| {
      error!("HTTP 请求失败: {}", e);
      NodeExecutionError::ExternalServiceError { service: "HTTP Request".to_string() }
    })?;

    let duration = start_time.elapsed();
    let status = response.status();
    let response_headers = response.headers().clone();

    debug!("HTTP 响应: status={}, duration={:?}", status, duration);

    // Get http response content type
    let content_type = response_headers
      .get("content-type")
      .and_then(|ct| ct.to_str().ok())
      .unwrap_or("application/octet-stream");

    // Process http response
    let response_data = if content_type.contains("application/json") || content_type.contains("text/") {
      // JSON 或文本内容
      let text = response.text().await.map_err(|e| {
        error!("Read response text failed: {}", e);
        NodeExecutionError::DataProcessingError { message: format!("Failed to read response text: {}", e) }
      })?;

      if content_type.contains("application/json") {
        // Try to parse as JSON
        match serde_json::from_str::<Value>(&text) {
          Ok(json) => json,
          Err(_) => {
            // JSON parsing failed, treat as plain text
            Value::String(text)
          }
        }
      } else {
        Value::String(text)
      }
    } else {
      // Binary content, return basic info
      let bytes = response.bytes().await.map_err(|e| {
        error!("Read response bytes failed: {}", e);
        NodeExecutionError::DataProcessingError { message: format!("Failed to read response bytes: {}", e) }
      })?;

      serde_json::json!({
        "content_type": content_type,
        "size": bytes.len(),
        "data": "[Binary Data]"
      })
    };

    // Build complete response data
    let result_data = serde_json::json!({
      "status": status.as_u16(),
      "status_text": status.canonical_reason().unwrap_or("Unknown"),
      "headers": response_headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.to_str().unwrap_or("")))
        .collect::<std::collections::HashMap<_, _>>(),
      "data": response_data,
      "url": url,
      "method": method,
      "duration_ms": duration.as_millis(),
      "content_type": content_type,
    });

    info!("HTTP request completed: status={}, duration={:?}", status, duration);

    let result = vec![ExecutionData::new_json(
      result_data,
      Some(DataSource {
        node_name: context.current_node_name.clone(),
        output_port: ConnectionKind::Main,
        output_index: 0,
      }),
    )];

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(result)])]))
  }
}

/// Create node definition
pub(super) fn create_definition() -> Result<NodeDefinition, RegistrationError> {
  let mut definition = NodeDefinition::new(HTTP_REQUEST_NODE_KIND, "HTTP Request")
    .add_group(NodeGroupKind::Input)
    .add_group(NodeGroupKind::Output)
    .with_description("发送HTTP请求并获取响应数据。支持GET、POST、PUT、DELETE等方法。")
    .with_icon("globe");

  // Add input port
  definition = definition.add_input(InputPortConfig::new(ConnectionKind::Main, "Input"));

  // Add output port
  definition = definition.add_output(OutputPortConfig::new(ConnectionKind::Main, "Output"));

  // Add properties
  definition = definition.add_property(
    NodeProperty::new(NodePropertyKind::String)
      .with_name("url")
      .with_display_name("URL")
      .with_required(true)
      .with_description("请求的目标URL地址")
      .with_placeholder("https://api.example.com"),
  );

  definition = definition.add_property(
    NodeProperty::new(NodePropertyKind::Options)
      .with_name("method")
      .with_display_name("Method")
      .with_required(true)
      .with_description("HTTP请求方法")
      .with_value(json!(HttpMethod::Get))
      .with_options(
        HttpMethod::ALL
          .iter()
          .map(|m| Box::new(NodeProperty::new_option(m.as_ref(), m.as_ref(), json!(m), NodePropertyKind::Options)))
          .collect::<Vec<_>>(),
      ),
  );

  definition = definition.add_property(
    NodeProperty::new(NodePropertyKind::String)
      .with_name("headers")
      .with_display_name("Headers")
      .with_description("HTTP请求头，JSON格式")
      .with_placeholder("{\"Content-Type\": \"application/json\"}"),
  );

  definition = definition.add_property(
    NodeProperty::new(NodePropertyKind::String)
      .with_name("body")
      .with_display_name("Body")
      .with_description("请求体内容，支持JSON格式")
      .with_placeholder("{\"key\": \"value\"}"),
  );

  definition = definition.add_property(
    NodeProperty::new(NodePropertyKind::Number)
      .with_name("timeout")
      .with_display_name("Timeout")
      .with_description("请求超时时间（秒），范围1-300")
      .with_value(json!(30))
      .with_placeholder("30"),
  );

  definition = definition.add_property(
    NodeProperty::new(NodePropertyKind::Boolean)
      .with_name("follow_redirects")
      .with_display_name("Follow Redirects")
      .with_description("是否跟随重定向")
      .with_value(json!(true)),
  );

  Ok(definition)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_node_definition() {
    let definition = create_definition().unwrap();
    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::HttpRequest");
    assert_eq!(definition.version, Version::new(1, 0, 0));
    assert_eq!(definition.groups, [NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(definition.display_name, "HTTP Request");
    assert!(!definition.properties.is_empty());
  }

  #[test]
  fn test_node_ports() {
    let node = HttpRequestV1::try_from(create_definition().unwrap()).unwrap();

    let input_ports = &node.definition().inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &node.definition().outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }
}
