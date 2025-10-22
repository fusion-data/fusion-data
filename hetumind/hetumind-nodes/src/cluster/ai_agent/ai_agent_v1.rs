use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use async_trait::async_trait;
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, NodeDefinition, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeSupplier, RegistrationError, make_execution_data_map,
};
use mea::rwlock::RwLock;
use rig::{client::CompletionClient, completion::Prompt};
use serde_json::json;
use uuid::Uuid;

use crate::cluster::ai_agent::parameters::ToolExecutionStatus;
use crate::cluster::ai_agent::tool_manager::ToolManager;
use crate::cluster::ai_agent::utils::create_base_definition;
use crate::core::connection_manager::OptimizedConnectionContext;

use super::parameters::{AiAgentConfig, ModelInstance, ToolCallRequest, ToolCallResult};

pub struct AiAgentV1 {
  pub definition: Arc<NodeDefinition>,
  #[allow(dead_code)]
  tool_manager: Arc<RwLock<ToolManager>>,
}

impl AiAgentV1 {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base_definition();
    Self::try_from(base)
  }
}

impl TryFrom<NodeDefinition> for AiAgentV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base.with_version(Version::new(1, 0, 0));
    Ok(Self { definition: Arc::new(definition), tool_manager: Arc::new(RwLock::new(ToolManager::new())) })
  }
}

#[async_trait]
impl NodeExecutable for AiAgentV1 {
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 1. 获取输入数据和配置
    let input_data = context.get_input_data(ConnectionKind::Main)?;
    let config: AiAgentConfig = context.get_parameters()?;

    // 2. 获取连接的 LLM 实例
    let llm_instance = self.get_llm_instance(context).await?;

    // 3. 获取连接的工具
    let tools = self.get_tools(context).await?;

    // 4. 创建 Agent
    let agent = self.create_agent(llm_instance, tools, &config).await?;

    // 5. 执行 Agent
    let result = if config.enable_streaming() {
      self.execute_agent_streaming(&agent, &input_data, &config).await?
    } else {
      self.execute_agent(&agent, &input_data, &config).await?
    };

    // 8. 返回最终结果
    Ok(make_execution_data_map(vec![(
      ConnectionKind::Main,
      vec![ExecutionDataItems::Items(vec![ExecutionData::new_json(
        json!({
            "response": result,
            "node_kind": &self.definition().kind,
            "streaming": config.enable_streaming,
            "timestamp": now_offset(),
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
  async fn get_llm_instance(&self, context: &NodeExecutionContext) -> Result<NodeSupplier, NodeExecutionError> {
    // TODO 获取 ConnectionKind::AiLM 的 Arc<dyn NodeSupplier>
    let lm_conn = context
      .workflow
      .connections
      .get(context.current_node_name())
      .and_then(|kind_conns| kind_conns.get(&ConnectionKind::AiLM))
      .and_then(|conns| conns.iter().next())
      .ok_or_else(|| {
        NodeExecutionError::ConfigurationError(format!(
          "No ConnectionKind::AiLM found, node_name: {}",
          context.current_node_name()
        ))
      })?;
    let node = context.workflow.get_node(lm_conn.node_name()).ok_or_else(|| {
      NodeExecutionError::ConnectionError(format!("No Node fount, node_name: {}", lm_conn.node_name()))
    })?;
    let lm = context.node_registry.get_supplier(&node.kind).ok_or_else(|| {
      NodeExecutionError::ConfigurationError(format!("No NodeSupplier found, node_kind: {}", lm_conn.kind()))
    })?;

    Ok(lm)
  }

  async fn get_tools(&self, context: &NodeExecutionContext) -> Result<Vec<JsonValue>, NodeExecutionError> {
    // 获取所有连接的工具（使用优化的批量获取）
    let tool_connections = context.get_all_connections_data(ConnectionKind::AiTool).await?;

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
    if let EngineAction::ExecuteNode(node_action) = &result.action
      && let Some(data) = result.data.get(&ConnectionKind::AiTool)
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
