//! HTTP 请求节点实现

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
  NodeExecutionContext, NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, NodeProperties, NodePropertyKind,
  OutputPortConfig, make_execution_data_map,
};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde_json::{Value, json};

use super::HttpMethod;

/// HTTP 请求节点
///
/// 用于发送 HTTP 请求并获取响应数据。支持 GET、POST、PUT、DELETE 等常见方法。
///
/// # 参数
/// - `url`: 请求的目标 URL
/// - `method`: HTTP 方法 (GET, POST, PUT, DELETE, PATCH)
/// - `headers`: 请求头 (可选)
/// - `body`: 请求体 (可选，适用于 POST/PUT/PATCH)
/// - `timeout`: 超时时间，秒 (可选，默认 30 秒)
/// - `follow_redirects`: 是否跟随重定向 (可选，默认 true)
/// - `max_redirects`: 最大重定向次数 (可选，默认 10)
#[derive(Debug, Clone)]
pub struct HttpRequestNode {
  definition: Arc<NodeDefinition>,
}

impl Default for HttpRequestNode {
  fn default() -> Self {
    Self { definition: Arc::new(create_definition()) }
  }
}

#[async_trait]
impl NodeExecutor for HttpRequestNode {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    info!(
      "开始执行 HTTP 请求节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // 获取必需参数
    let url = node.parameters.get_parameter::<String>("url")?;
    let method = node.parameters.get_parameter::<HttpMethod>("method").unwrap_or(HttpMethod::Get);

    // 获取可选参数
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

    // 构建请求
    let mut request_builder = method.create_request_builder(&client, &url);

    // 添加请求头
    for (key, value) in headers {
      if let Some(header_value) = value.as_str() {
        request_builder = request_builder.header(&key, header_value);
      }
    }

    // 添加请求体（如果存在且方法支持）
    if let Some(body_data) = body {
      if matches!(method, HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch) {
        request_builder = request_builder.json(&body_data);
      } else {
        warn!("HTTP 方法 {:?} 不支持请求体，忽略 body 参数", method);
      }
    }

    // 发送请求
    let start_time = std::time::Instant::now();
    let response = request_builder.send().await.map_err(|e| {
      error!("HTTP 请求失败: {}", e);
      NodeExecutionError::ExternalServiceError { service: "HTTP Request".to_string() }
    })?;

    let duration = start_time.elapsed();
    let status = response.status();
    let response_headers = response.headers().clone();

    debug!("HTTP 响应: status={}, duration={:?}", status, duration);

    // 获取响应内容类型
    let content_type = response_headers
      .get("content-type")
      .and_then(|ct| ct.to_str().ok())
      .unwrap_or("application/octet-stream");

    // 处理响应
    let response_data = if content_type.contains("application/json") || content_type.contains("text/") {
      // JSON 或文本内容
      let text = response.text().await.map_err(|e| {
        error!("读取响应内容失败: {}", e);
        NodeExecutionError::DataProcessingError { message: format!("Failed to read response text: {}", e) }
      })?;

      if content_type.contains("application/json") {
        // 尝试解析为 JSON
        match serde_json::from_str::<Value>(&text) {
          Ok(json) => json,
          Err(_) => {
            // JSON 解析失败，作为文本处理
            Value::String(text)
          }
        }
      } else {
        Value::String(text)
      }
    } else {
      // 二进制内容，返回基本信息
      let bytes = response.bytes().await.map_err(|e| {
        error!("读取响应字节失败: {}", e);
        NodeExecutionError::DataProcessingError { message: format!("Failed to read response bytes: {}", e) }
      })?;

      serde_json::json!({
        "content_type": content_type,
        "size": bytes.len(),
        "data": "[Binary Data]"
      })
    };

    // 构建完整的响应数据
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

    info!("HTTP 请求完成: status={}, duration={:?}", status, duration);

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

impl HttpRequestNode {
  pub const NODE_KIND: &str = "HttpRequest";
}

/// 获取节点元数据
fn create_definition() -> NodeDefinition {
  NodeDefinition::builder()
    .kind(NodeKind::from(HttpRequestNode::NODE_KIND))
    .versions(vec![1])
    .groups(vec![NodeGroupKind::Input, NodeGroupKind::Output])
    .display_name("HTTP Request")
    .description("发送HTTP请求并获取响应数据。支持GET、POST、PUT、DELETE等方法。")
    .icon("globe")
    .inputs(vec![InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
    .outputs(vec![OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
    .properties(vec![
      NodeProperties::builder()
        .name("url".to_string())
        .kind(NodePropertyKind::String)
        .required(true)
        .display_name("URL")
        .description("请求的目标URL地址")
        .placeholder("https://api.example.com")
        .build(),
      NodeProperties::builder()
        .name("method".to_string())
        .kind(NodePropertyKind::Options)
        .required(true)
        .display_name("Method")
        .description("HTTP请求方法")
        .value(json!(HttpMethod::Get))
        .options(
          HttpMethod::ALL
            .iter()
            .map(|m| Box::new(NodeProperties::new_option(m.as_ref(), m.as_ref(), json!(m), NodePropertyKind::Options)))
            .collect(),
        )
        .build(),
      NodeProperties::builder()
        .name("headers".to_string())
        .kind(NodePropertyKind::String)
        .required(false)
        .display_name("Headers")
        .description("HTTP请求头，JSON格式")
        .placeholder("{\"Content-Type\": \"application/json\"}")
        .build(),
      NodeProperties::builder()
        .name("body".to_string())
        .kind(NodePropertyKind::String)
        .required(false)
        .display_name("Body")
        .description("请求体内容，支持JSON格式")
        .placeholder("{\"key\": \"value\"}")
        .build(),
      NodeProperties::builder()
        .name("timeout".to_string())
        .kind(NodePropertyKind::Number)
        .required(false)
        .display_name("Timeout")
        .description("请求超时时间（秒），范围1-300")
        .value(json!(30))
        .placeholder("30")
        .build(),
    ])
    .build()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_node_metadata() {
    let metadata = create_definition();
    assert_eq!(metadata.kind.as_ref(), "HttpRequest");
    assert_eq!(metadata.default_version, None);
    assert_eq!(metadata.versions, vec![1]);
    assert_eq!(&metadata.groups, &[NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&metadata.display_name, "HTTP Request");
    assert!(!metadata.properties.is_empty());
  }

  #[test]
  fn test_node_ports() {
    let node = HttpRequestNode::default();

    let input_ports = &node.definition().inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &node.definition().outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }
}
