use async_trait::async_trait;
use hetumind_core::workflow::NodeExecutionError;
use rig::message::ToolResult;
use serde_json::Value;
use std::collections::HashMap;

/// 工具定义结构
#[derive(Debug, Clone)]
pub struct ToolDefinition {
  pub name: String,
  pub description: String,
  pub parameters: Value,
}

/// 工具管理器，负责注册和管理AI Agent可用的工具
pub struct ToolManager {
  tools: HashMap<String, ToolDefinition>,
}

impl ToolManager {
  /// 创建新的工具管理器
  pub fn new() -> Self {
    Self { tools: HashMap::new() }
  }

  /// 注册工具定义
  pub fn register_tool(&mut self, name: String, tool: ToolDefinition) {
    self.tools.insert(name, tool);
  }

  /// 获取工具定义
  pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
    self.tools.get(name)
  }

  /// 获取所有工具名称
  pub fn list_tools(&self) -> Vec<String> {
    self.tools.keys().cloned().collect()
  }

  /// 将工具定义转换为 rig-core 格式
  pub async fn convert_tool_definition(&self, tool_def: &Value) -> Result<ToolDefinition, NodeExecutionError> {
    let tool_name = tool_def
      .get("name")
      .and_then(|v| v.as_str())
      .ok_or_else(|| NodeExecutionError::ConfigurationError("Tool name missing".to_string()))?
      .to_string();

    let tool_description = tool_def
      .get("description")
      .and_then(|v| v.as_str())
      .unwrap_or("A dynamically created tool")
      .to_string();

    let tool_parameters = tool_def.get("parameters").cloned().unwrap_or(Value::Object(serde_json::Map::new()));

    Ok(ToolDefinition { name: tool_name, description: tool_description, parameters: tool_parameters })
  }
}

impl Default for ToolManager {
  fn default() -> Self {
    Self::new()
  }
}

/// 工具调用结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionResult {
  /// 工具调用ID
  pub call_id: String,
  /// 工具名称
  pub tool_name: String,
  /// 执行结果
  pub result: Value,
  /// 是否成功
  pub success: bool,
  /// 错误信息（如果有）
  pub error: Option<String>,
  /// 执行时间（毫秒）
  pub duration_ms: u64,
}

impl ToolExecutionResult {
  /// 创建成功的执行结果
  pub fn success(call_id: String, tool_name: String, result: Value, duration_ms: u64) -> Self {
    Self { call_id, tool_name, result, success: true, error: None, duration_ms }
  }

  /// 创建失败的执行结果
  pub fn failure(call_id: String, tool_name: String, error: String, duration_ms: u64) -> Self {
    Self { call_id, tool_name, result: Value::Null, success: false, error: Some(error), duration_ms }
  }
}
