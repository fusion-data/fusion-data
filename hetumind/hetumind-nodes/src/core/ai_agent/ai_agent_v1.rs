use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, EngineAction, EngineRequest, EngineResponse, ExecuteNodeAction, ExecutionData, ExecutionDataItems,
    ExecutionDataMap, InputPortConfig, NodeDefinition, NodeDefinitionBuilder, NodeExecutable, NodeExecutionContext,
    NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig, RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;
use uuid::Uuid;

use crate::core::ai_agent::parameters::ToolExecutionStatus;

use super::parameters::{AiAgentConfig, ModelInstance, ToolCallRequest, ToolCallResult};

#[derive(Debug)]
pub struct AiAgentV1 {
  pub definition: Arc<NodeDefinition>,
}

impl AiAgentV1 {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = NodeDefinitionBuilder::default();
    Self::try_from(base)
  }
}

impl TryFrom<NodeDefinitionBuilder> for AiAgentV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .kind(hetumind_core::workflow::NodeKind::from("ai_agent"))
      .version(hetumind_core::version::Version::new(1, 0, 0))
      .display_name("AI Agent")
      .description("AI Agent 节点，支持工具调用和记忆功能")
      .icon("🤖")

      // 输入端口
      .inputs([
        InputPortConfig::builder()
          .kind(ConnectionKind::Main)
          .display_name("Main Input")
          .required(true)
          .build(),
        InputPortConfig::builder()
          .kind(ConnectionKind::AiLanguageModel)
          .display_name("Large Language Model")
          .required(true)
          .max_connections(1)
          .build(),
        InputPortConfig::builder()
          .kind(ConnectionKind::AiMemory)
          .display_name("Memory(Vector storage)")
          .required(false)
          .build(),
        InputPortConfig::builder()
          .kind(ConnectionKind::AiTool)
          .display_name("AI Tool")
          .required(false)
          .build(),
      ])

      // 输出端口
      .outputs([
          OutputPortConfig::builder()
            .kind(ConnectionKind::Main)
            .display_name("AI 响应输出")
            .build(),
          OutputPortConfig::builder()
            .kind(ConnectionKind::AiTool)
            .display_name("工具调用请求")
            .build(),
          OutputPortConfig::builder()
            .kind(ConnectionKind::Error)
            .display_name("错误输出")
            .build(),
      ])

      // 参数
      .properties([
        NodeProperty::builder()
          .display_name("系统提示词")
          .name("system_prompt")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("AI Agent 的系统提示词")
          .value(json!("你是一个有帮助的AI助手"))
          .build(),
        NodeProperty::builder()
          .display_name("最大迭代次数")
          .name("max_iterations")
          .kind(NodePropertyKind::Number)
          .required(false)
          .description("AI Agent 的最大迭代次数")
          .value(json!(10))
          .build(),
        NodeProperty::builder()
          .display_name("温度参数")
          .name("temperature")
          .kind(NodePropertyKind::Number)
          .required(false)
          .description("控制生成文本的随机性")
          .value(json!(0.7))
          .build(),
        NodeProperty::builder()
          .display_name("是否启用流式响应")
          .name("enable_streaming")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("是否启用流式响应")
          .value(json!(false))
          .build(),
      ]);

    let definition = base.build()?;
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for AiAgentV1 {
  async fn execute(
    &self,
    context: &NodeExecutionContext,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 1. 获取输入数据和配置
    let input_data = context.get_input_data("main")?;
    let config: AiAgentConfig = context.get_parameters()?;

    // 2. 处理引擎响应（工具调用结果）
    if let Some(response) = &context.engine_response {
      return self.handle_tool_responses(context, response, &config).await;
    }

    // 3. 获取连接的 LLM 实例
    let llm_instance = self.get_llm_instance(context).await?;

    // 4. 获取连接的工具
    let tools = self.get_tools(context).await?;

    // 5. 创建 Agent
    let agent = self.create_agent(llm_instance, tools, &config).await?;

    // 6. 执行 Agent
    let result = self.execute_agent(&agent, &input_data, &config).await?;

    // 7. 解析响应，检查是否需要工具调用
    if let Some(tool_calls) = self.parse_tool_calls(&result) {
      // 返回引擎请求以执行工具
      return self.create_engine_request(context, tool_calls, &config).await;
    }

    // 8. 返回最终结果
    Ok(make_execution_data_map(vec![(
      ConnectionKind::Main,
      vec![ExecutionDataItems::Items(vec![ExecutionData::new_json(
        json!({
            "response": result,
            "agent_type": "ai_agent_v1",
            "timestamp": chrono::Utc::now().timestamp(),
        }),
        None,
      )])],
    )]))
  }

  fn definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.definition)
  }
}

impl AiAgentV1 {
  async fn get_llm_instance(&self, context: &NodeExecutionContext) -> Result<ModelInstance, NodeExecutionError> {
    // 通过连接类型获取 LLM 实例
    let connection_data = context
      .get_connection_data(ConnectionKind::AiLanguageModel, 0)
      .ok_or_else(|| NodeExecutionError::ConnectionError("No LLM model connected".to_string()))?;

    // 解析 LLM 实例
    self.parse_llm_instance(connection_data)
  }

  fn parse_llm_instance(&self, connection_data: ExecutionData) -> Result<ModelInstance, NodeExecutionError> {
    let data = connection_data.json();
    serde_json::from_value(data.clone())
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("Failed to parse LLM instance: {}", e)))
  }

  async fn get_tools(&self, context: &NodeExecutionContext) -> Result<Vec<JsonValue>, NodeExecutionError> {
    // 获取所有连接的工具
    let tool_connections = context.get_all_connections(ConnectionKind::AiTool);

    let mut tools = Vec::new();
    for connection in tool_connections {
      tools.push(connection.json().clone());
    }

    Ok(tools)
  }

  async fn create_agent(
    &self,
    llm: ModelInstance,
    tools: Vec<JsonValue>,
    config: &AiAgentConfig,
  ) -> Result<JsonValue, NodeExecutionError> {
    // 创建 Agent 配置
    Ok(json!({
        "llm": llm,
        "tools": tools,
        "system_prompt": config.system_prompt,
        "max_iterations": config.max_iterations,
        "temperature": config.temperature,
    }))
  }

  async fn execute_agent(
    &self,
    agent: &JsonValue,
    input_data: &ExecutionData,
    config: &AiAgentConfig,
  ) -> Result<String, NodeExecutionError> {
    // 模拟 Agent 执行，实际实现需要集成 rig-core
    let prompt = input_data.json().get("prompt").and_then(|v| v.as_str()).unwrap_or("请处理这个请求");

    // 这里应该使用 rig-core 的 Agent 执行
    // 目前返回模拟响应
    Ok(format!("AI Agent 响应: {}", prompt))
  }

  fn parse_tool_calls(&self, result: &str) -> Option<Vec<ToolCallRequest>> {
    // 解析工具调用（这里需要实现实际的解析逻辑）
    // 目前返回空列表，表示没有工具调用
    None
  }

  async fn handle_tool_responses(
    &self,
    context: &NodeExecutionContext,
    response: &EngineResponse,
    config: &AiAgentConfig,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 处理工具执行结果，继续对话
    let tool_results: Vec<ToolCallResult> =
      response.action_responses.iter().filter_map(|ar| self.extract_tool_result(ar)).collect();

    // 构建包含工具结果的提示
    let prompt = self.build_prompt_with_tool_results(context, &tool_results, config).await?;

    // 获取 Agent 并执行
    let llm_instance = self.get_llm_instance(context).await?;
    let tools = self.get_tools(context).await?;
    let agent = self.create_agent(llm_instance, tools, config).await?;

    let input_data = ExecutionData::new_json(json!({"prompt": prompt}), None);
    let final_response = self.execute_agent(&agent, &input_data, config).await?;

    Ok(make_execution_data_map(vec![(
      ConnectionKind::Main,
      vec![ExecutionDataItems::Items(vec![ExecutionData::new_json(
        json!({
            "response": final_response,
            "tool_results": tool_results,
            "agent_type": "ai_agent_v1",
            "timestamp": chrono::Utc::now().timestamp(),
        }),
        None,
      )])],
    )]))
  }

  fn extract_tool_result(&self, result: &hetumind_core::workflow::EngineResult) -> Option<ToolCallResult> {
    // 从引擎结果中提取工具调用结果
    match &result.action {
      EngineAction::ExecuteNode(node_action) => {
        if let Some(data) = result.data.get(&ConnectionKind::AiTool) {
          if let Some(items) = data.first() {
            if let Some(data_items) = items.get_data_items() {
              if let Some(execution_data) = data_items.first() {
                return Some(ToolCallResult {
                  tool_call_id: node_action.action_id.to_string(),
                  tool_name: node_action.node_name.clone(),
                  result: execution_data.json().clone(),
                  status: ToolExecutionStatus::Success,
                });
              }
            }
          }
        }
      }
      _ => {}
    }
    None
  }

  async fn build_prompt_with_tool_results(
    &self,
    _context: &NodeExecutionContext,
    tool_results: &Vec<ToolCallResult>,
    _config: &AiAgentConfig,
  ) -> Result<String, NodeExecutionError> {
    // 构建包含工具结果的提示
    let mut prompt = String::new();
    for result in tool_results {
      prompt.push_str(&format!("工具 {} 执行结果: {:?}\n", result.tool_name, result.result));
    }
    Ok(prompt)
  }

  async fn create_engine_request(
    &self,
    _context: &NodeExecutionContext,
    tool_calls: Vec<ToolCallRequest>,
    config: &AiAgentConfig,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    let actions: Vec<EngineAction> = tool_calls
      .into_iter()
      .map(|tool_call| {
        let tool_name = tool_call.tool_name.clone();
        EngineAction::ExecuteNode(ExecuteNodeAction {
          node_name: tool_call.tool_name,
          input: tool_call.parameters,
          connection_type: ConnectionKind::AiTool,
          action_id: Uuid::new_v4(),
          metadata: {
            let mut meta = HashMap::new();
            meta.insert("tool_call_id".to_string(), json!(tool_call.id));
            meta.insert("tool_name".to_string(), json!(tool_name));
            meta
          },
        })
      })
      .collect();

    let engine_request = EngineRequest {
      actions,
      metadata: {
        let mut meta = HashMap::new();
        meta.insert("request_type".to_string(), json!("tool_execution"));
        meta.insert("config".to_string(), json!(config));
        meta
      },
      request_id: Uuid::new_v4(),
    };

    Ok(make_execution_data_map(vec![(
      ConnectionKind::AiTool,
      vec![ExecutionDataItems::Items(vec![ExecutionData::new_json(json!(engine_request), None)])],
    )]))
  }
}
