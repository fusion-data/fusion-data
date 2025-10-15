use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, EngineAction, EngineRequest, EngineResponse, ExecuteNodeAction, ExecutionData, ExecutionDataItems,
    ExecutionDataMap, InputPortConfig, NodeDefinition, NodeExecutable, NodeExecutionContext, NodeExecutionError,
    NodeProperty, NodePropertyKind, OutputPortConfig, RegistrationError, make_execution_data_map,
  },
};
use rig::{client::CompletionClient, completion::Prompt};
use serde_json::json;
use uuid::Uuid;

use crate::core::ai_agent::tool_manager::ToolManager;
use crate::core::connection_manager::OptimizedConnectionContext;
use crate::{constants::AI_AGENT_NODE_KIND, core::ai_agent::parameters::ToolExecutionStatus};

use super::parameters::{AiAgentConfig, ModelInstance, ToolCallRequest, ToolCallResult};

#[allow(dead_code)]
pub struct AiAgentV1 {
  pub definition: Arc<NodeDefinition>,
  #[allow(dead_code)]
  tool_manager: Arc<tokio::sync::RwLock<ToolManager>>,
}

impl AiAgentV1 {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = NodeDefinition::new(AI_AGENT_NODE_KIND, "AI Agent");
    Self::try_from(base)
  }
}

impl TryFrom<NodeDefinition> for AiAgentV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .with_description("AI Agent 节点，支持工具调用和记忆功能")
      .with_icon("🤖")
      // 输入端口
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Main Input")
          .with_required(true))
      .add_input(InputPortConfig::new(ConnectionKind::AiModel, "Large Language Model")
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
          .with_value(json!(false)));

    Ok(Self { definition: Arc::new(definition), tool_manager: Arc::new(tokio::sync::RwLock::new(ToolManager::new())) })
  }
}

#[async_trait]
impl NodeExecutable for AiAgentV1 {
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
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
    let result = if config.enable_streaming {
      self.execute_agent_streaming(&agent, &input_data, &config).await?
    } else {
      self.execute_agent(&agent, &input_data, &config).await?
    };

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
            "streaming": config.enable_streaming,
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
    // 通过优化的连接类型获取 LLM 实例
    let connection_data = context
      .get_connection_data_optimized(ConnectionKind::AiModel, 0)
      .await?
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
    // 获取所有连接的工具（使用优化的批量获取）
    let tool_connections = context.get_all_connections_optimized(ConnectionKind::AiTool).await?;

    let mut tools = Vec::new();
    for connection in tool_connections {
      tools.push(connection.json().clone());
    }

    Ok(tools)
  }

  #[allow(unused_variables)]
  async fn create_agent(
    &self,
    llm_instance: ModelInstance,
    _tools: Vec<JsonValue>,
    config: &AiAgentConfig,
  ) -> Result<Box<dyn std::any::Any + Send + Sync>, NodeExecutionError> {
    // 创建 rig-core Agent
    let agent = self.create_rig_agent(&llm_instance).await?;
    Ok(Box::new(agent))
  }

  async fn execute_agent(
    &self,
    agent: &Box<dyn std::any::Any + Send + Sync>,
    input_data: &ExecutionData,
    _config: &AiAgentConfig,
  ) -> Result<String, NodeExecutionError> {
    let prompt = input_data
      .json()
      .get("prompt")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string())
      .unwrap_or_else(|| input_data.json().to_string());

    // 尝试转换为不同类型的 Agent
    if let Some(openai_agent) =
      agent.downcast_ref::<rig::agent::Agent<rig::providers::openai::completion::CompletionModel>>()
    {
      openai_agent.prompt(prompt).await.map_err(|e| NodeExecutionError::ExecutionFailed {
        node_name: "AiAgentV1".to_string().into(),
        message: Some(format!("OpenAI API call failed: {}", e)),
      })
    } else if let Some(anthropic_agent) =
      agent.downcast_ref::<rig::agent::Agent<rig::providers::anthropic::completion::CompletionModel>>()
    {
      anthropic_agent.prompt(prompt).await.map_err(|_| NodeExecutionError::ExecutionFailed {
        node_name: "AiAgentV1".to_string().into(),
        message: Some("Anthropic API call failed".to_string()),
      })
    } else {
      Err(NodeExecutionError::ExecutionFailed {
        node_name: "AiAgentV1".to_string().into(),
        message: Some("Unsupported AI Agent type".to_string()),
      })
    }
  }

  #[allow(unused_variables)]
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
        if let Some(data) = result.data.get(&ConnectionKind::AiTool)
          && let Some(items) = data.first()
          && let Some(data_items) = items.get_data_items()
          && let Some(execution_data) = data_items.first()
        {
          return Some(ToolCallResult {
            tool_call_id: node_action.action_id.to_string(),
            tool_name: node_action.node_name.clone(),
            result: execution_data.json().clone(),
            status: ToolExecutionStatus::Success,
          });
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

  /// 创建 rig-core Agent 实例
  async fn create_rig_agent(
    &self,
    llm_instance: &ModelInstance,
  ) -> Result<Box<dyn std::any::Any + Send + Sync>, NodeExecutionError> {
    // 从ModelInstance的config中获取LLM配置
    let config = &llm_instance.config;

    let provider = config.get("provider").and_then(|v| v.as_str()).unwrap_or("openai");

    match provider {
      "openai" => {
        use rig::providers::openai;

        let api_key = config
          .get("api_key")
          .and_then(|v| v.as_str())
          .ok_or_else(|| NodeExecutionError::ConfigurationError("OpenAI API key not found".to_string()))?;

        let client = openai::Client::new(api_key);

        // 根据文档示例，直接使用 client.agent() 方法创建
        let model_name = config.get("model").and_then(|v| v.as_str()).unwrap_or("gpt-3.5-turbo");

        let agent = client.agent(model_name);
        Ok(Box::new(agent))
      }
      "anthropic" => {
        use rig::providers::anthropic;

        let api_key = config
          .get("api_key")
          .and_then(|v| v.as_str())
          .ok_or_else(|| NodeExecutionError::ConfigurationError("Anthropic API key not found".to_string()))?;

        let client = anthropic::Client::new(api_key);

        let model_name = config.get("model").and_then(|v| v.as_str()).unwrap_or("claude-3-sonnet-20240229");

        let agent = client.agent(model_name);
        Ok(Box::new(agent))
      }
      _ => Err(NodeExecutionError::ConfigurationError(format!("Unsupported LLM provider: {}", provider))),
    }
  }

  /// 将工具定义转换为 rig-core 格式（暂时不实现）
  #[allow(dead_code)]
  async fn convert_to_rig_tool(&self, _tool: JsonValue) -> Result<String, NodeExecutionError> {
    // TODO: 暂时不实现工具转换，直接返回工具名称
    Ok("tool_conversion_not_implemented".to_string())
  }

  /// 流式执行Agent
  #[allow(unused_variables)]
  async fn execute_agent_streaming(
    &self,
    agent: &Box<dyn std::any::Any + Send + Sync>,
    input_data: &ExecutionData,
    config: &AiAgentConfig,
  ) -> Result<String, NodeExecutionError> {
    let prompt = input_data
      .json()
      .get("prompt")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string())
      .unwrap_or_else(|| input_data.json().to_string());

    // 模拟流式Agent执行
    let mut streaming_response = String::new();

    // 模拟AI Agent思考过程的流式输出
    let thinking_steps = vec!["正在分析用户请求...", "理解问题上下文...", "准备生成响应..."];

    for step in thinking_steps {
      streaming_response.push_str(&format!("[{}]\n", step));

      // 模拟思考时间
      tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    // 执行实际的Agent推理
    let final_response = self.execute_agent(agent, input_data, config).await?;
    streaming_response.push_str(&final_response);

    Ok(streaming_response)
  }
}
