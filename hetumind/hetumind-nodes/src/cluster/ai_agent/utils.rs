use hetumind_core::types::JsonValue;
use hetumind_core::workflow::{
  ConnectionKind, InputPortConfig, NodeDefinition, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
};
use serde_json::json;

use crate::constants::AI_AGENT_NODE_KIND;

pub fn create_base_definition() -> NodeDefinition {
  NodeDefinition::new(AI_AGENT_NODE_KIND, "AI Agent")
    .with_description("AI Agent 节点，支持工具调用和记忆功能")
    .with_icon("🤖")

    // 参数
    .add_property(NodeProperty::new(NodePropertyKind::String)
        .with_display_name("系统提示词")
        .with_name("system_prompt")
        .with_required(false)
        .with_description("AI Agent 的系统提示词")
        .with_value(json!("你是一个有帮助的AI助手")))
    .add_property(NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("最大迭代次数")
        .with_name("max_iterations")
        .with_required(false)
        .with_description("AI Agent 的最大迭代次数")
        .with_value(json!(10)))
    .add_property(NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("温度参数")
        .with_name("temperature")
        .with_required(false)
        .with_description("控制生成文本的随机性")
        .with_value(json!(0.7)))
    .add_property(NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("是否启用流式响应")
        .with_name("enable_streaming")
        .with_required(false)
        .with_description("是否启用流式响应")
        .with_value(json!(false)))

    // 输入端口
    .add_input(InputPortConfig::new(ConnectionKind::Main, "Main Input")
        .with_required(true))
    .add_input(InputPortConfig::new(ConnectionKind::AiLM, "Large Language Model")
        .with_required(true)
        .with_max_connections(1))
    .add_input(InputPortConfig::new(ConnectionKind::AiMemory, "Memory(Vector storage)")
        .with_required(false))
    .add_input(InputPortConfig::new(ConnectionKind::AiTool, "AI Tool")
        .with_required(false))

    // 输出端口
    .add_output(OutputPortConfig::new(ConnectionKind::Main, "AI 响应输出"))
    .add_output(OutputPortConfig::new(ConnectionKind::AiTool, "工具调用请求"))
    .add_output(OutputPortConfig::new(ConnectionKind::Error, "错误输出"))
}

/// 验证 JSON 结构
pub fn validate_json_structure(json: &JsonValue, required_fields: &[&str]) -> Result<(), NodeExecutionError> {
  if let JsonValue::Object(map) = json {
    for field in required_fields {
      if !map.contains_key(*field) {
        return Err(NodeExecutionError::InvalidInput(format!("Missing required field: {}", field)));
      }
    }
    Ok(())
  } else {
    Err(NodeExecutionError::InvalidInput("Expected JSON object".to_string()))
  }
}

/// 从 JSON 中提取字符串值
pub fn extract_string(json: &JsonValue, key: &str) -> Option<String> {
  json.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// 从 JSON 中提取数字值
pub fn extract_number(json: &JsonValue, key: &str) -> Option<serde_json::Number> {
  json.get(key).and_then(|v| v.as_number()).cloned()
}

/// 从 JSON 中提取布尔值
pub fn extract_bool(json: &JsonValue, key: &str) -> Option<bool> {
  json.get(key).and_then(|v| v.as_bool())
}

/// 构建错误消息
pub fn build_error_message(error: &str, context: &str) -> String {
  format!("{}: {}", context, error)
}

/// 解析工具调用参数
pub fn parse_tool_call_params(json: &JsonValue) -> Result<Vec<String>, NodeExecutionError> {
  match json {
    JsonValue::Array(arr) => {
      let params: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
      Ok(params)
    }
    JsonValue::String(s) => Ok(vec![s.clone()]),
    _ => Err(NodeExecutionError::InvalidInput("Expected string or array".to_string())),
  }
}

/// 格式化时间戳
pub fn format_timestamp(timestamp: i64) -> String {
  chrono::DateTime::from_timestamp(timestamp, 0)
    .unwrap_or_else(|| chrono::Utc::now())
    .format("%Y-%m-%d %H:%M:%S UTC")
    .to_string()
}

/// 创建响应数据结构
pub fn create_response_data(content: &str, metadata: Option<JsonValue>) -> JsonValue {
  let mut response = json!({
      "content": content,
      "timestamp": chrono::Utc::now().timestamp(),
  });

  if let Some(meta) = metadata {
    response["metadata"] = meta;
  }

  response
}

/// 验证工具调用格式
pub fn validate_tool_call_format(tool_call: &JsonValue) -> Result<(), NodeExecutionError> {
  validate_json_structure(tool_call, &["id", "tool_name", "parameters"])?;

  // 验证 ID 格式
  if let Some(id) = tool_call.get("id") {
    if !id.is_string() {
      return Err(NodeExecutionError::InvalidInput("Tool call ID must be a string".to_string()));
    }
  }

  // 验证工具名称格式
  if let Some(name) = tool_call.get("tool_name") {
    if !name.is_string() {
      return Err(NodeExecutionError::InvalidInput("Tool name must be a string".to_string()));
    }
  }

  Ok(())
}
