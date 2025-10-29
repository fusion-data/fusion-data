use std::sync::Arc;

use async_trait::async_trait;
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, ExecutionDataItems, ExecutionDataMap, NodeDefinition, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeSupplier, RegistrationError, SupplyResult,
};
use log::{debug, info, warn};
use mea::rwlock::RwLock;
use rig::message::{Message, UserContent};
use serde_json::json;

use crate::cluster::ai_agent::tool_manager::ToolManager;
use crate::cluster::ai_agent::utils::create_base_definition;
use crate::memory::simple_memory_node::{ConversationMessage, MessageRole, SimpleMemoryAccessor};

use super::parameters::AiAgentConfig;

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

    info!("Executing AiAgent with config: {:?}", config);

    // 2. 获取 memory （如果有的话）
    let memory_accessor = self.get_memory_accessor(context).await?;

    // 3. 从内存中获取历史消息
    let mut conversation_history = Vec::new();
    if let Some(memory) = &memory_accessor {
      let recent_messages =
        memory.get_recent_messages(config.memory_config().and_then(|c| c.context_window).unwrap_or(5));
      conversation_history = recent_messages.into_iter().cloned().collect();
      debug!("Retrieved {} messages from memory", conversation_history.len());
    }

    // 4. 获取连接的工具
    let _tools = self.get_tools(context).await?;

    // 5. 获取连接的 LLM 实例
    let llm = self.get_llm_instance(context).await?;

    // 6. 准备输入消息，包含历史对话
    let mut messages = Vec::new();

    // 系统提示词将在 LLM 调用中通过 preamble() 设置

    // 添加历史对话
    for msg in &conversation_history {
      match msg.role {
        MessageRole::System => {
          // 系统消息将在 preamble 中设置，这里跳过
          debug!("System message in history: {}", msg.content);
        }
        MessageRole::User => {
          messages.push(Message::user(msg.content.clone()));
        }
        MessageRole::Assistant => {
          messages.push(Message::assistant(msg.content.clone()));
        }
        MessageRole::Tool => {
          // 工具消息通常作为用户消息处理或特殊处理
          debug!("Tool message in history: {}", msg.content);
        }
      }
    }

    // 添加当前用户输入
    let current_input = input_data
      .get_value::<Message>("prompt")
      .map_err(|e| NodeExecutionError::invalid_input(format!("Get parameter 'prompt' failed, error: {}", e)))?;

    // 将当前输入添加到消息历史
    match &current_input {
      Message::User { content } => {
        let text = match content.first() {
          UserContent::Text(text) => text.text,
          _ => "".to_string(),
        };

        if text.starts_with("System:") {
          // 处理伪装成系统消息的用户消息
          let system_msg =
            ConversationMessage::new(MessageRole::System, text.trim_start_matches("System:").trim().to_string());
          if let Some(memory) = &memory_accessor {
            debug!("Would save system message to memory session: {}", memory.session_id);
          }
        } else {
          let user_msg = ConversationMessage::new(MessageRole::User, text);
          // 注意：在简化的架构中，我们不能直接保存回内存
          // 因为 SimpleMemoryNode 已经执行过了，内存数据是只读的
          // 在实际的工作流执行中，需要在下一轮执行时才能看到保存的消息
          if let Some(memory) = &memory_accessor {
            debug!("Would save user message to memory session: {}", memory.session_id);
          }
        }
      }
      _ => {
        debug!("Unsupported message type for memory storage");
      }
    }

    messages.push(current_input);

    // 7. 执行 LLM 调用
    let result = self.execute_llm_with_memory(&llm, &messages, &config, &memory_accessor).await?;

    // 8. 返回最终结果
    let mut data_map = ExecutionDataMap::default();
    data_map.insert(
      ConnectionKind::Main,
      vec![ExecutionDataItems::new_item(ExecutionData::new_json(
        json!({
            "response": result,
            "node_kind": &self.definition().kind,
            "streaming": config.enable_streaming,
            "timestamp": now_offset(),
            "memory_stats": {
              "history_length": conversation_history.len(),
              "has_memory": memory_accessor.is_some(),
              "session_id": memory_accessor.as_ref().map(|m| &m.session_id),
            }
        }),
        Some(DataSource::new(context.current_node_name().clone(), ConnectionKind::Main, 0)),
      ))],
    );

    info!("AiAgent execution completed successfully");
    Ok(data_map)
  }

  fn definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.definition)
  }
}

impl AiAgentV1 {
  /// 获取连接的 SimpleMemory 节点数据
  async fn get_memory_accessor(
    &self,
    context: &NodeExecutionContext,
  ) -> Result<Option<SimpleMemoryAccessor>, NodeExecutionError> {
    // 查找 ConnectionKind::AiMemory 的连接
    let memory_conn = context
      .workflow
      .connections
      .get(context.current_node_name())
      .and_then(|kind_conns| kind_conns.get(&ConnectionKind::AiMemory))
      .and_then(|conns| conns.iter().next());

    match memory_conn {
      Some(conn) => {
        info!("Found memory connection to node: {}", conn.node_name());

        // 执行 SimpleMemory 节点以获取内存数据
        let memory_node = context
          .workflow
          .get_node(conn.node_name())
          .ok_or_else(|| NodeExecutionError::ConnectionError(format!("Memory node not found: {}", conn.node_name())))?;

        let memory_executor = context.node_registry.get_executor(&memory_node.kind).ok_or_else(|| {
          NodeExecutionError::ConfigurationError(format!("No executor found for memory node: {}", memory_node.kind))
        })?;

        // 执行 SimpleMemory 节点
        let memory_data_map = memory_executor.execute(context).await?;
        let memory_data = memory_data_map
          .get(&ConnectionKind::AiMemory)
          .ok_or_else(|| NodeExecutionError::InvalidInput("Memory node did not return AiMemory data".to_string()))?;

        // 从内存数据中提取访问器
        if let Some(first_item) = memory_data.first()
          && let Some(items) = first_item.get_data_items()
          && let Some(execution_data) = items.first()
        {
          let accessor = SimpleMemoryAccessor::from_execution_data(execution_data)?;
          debug!("Created memory accessor for session: {}", accessor.session_id);
          return Ok(Some(accessor));
        }

        warn!("Memory node executed but no valid memory data found");
        Ok(None)
      }
      None => {
        debug!("No memory connection found for AiAgent");
        Ok(None)
      }
    }
  }

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

  async fn get_tools(&self, _context: &NodeExecutionContext) -> Result<Vec<JsonValue>, NodeExecutionError> {
    let tools = Vec::new();
    // TODO 获取所有连接的工具（使用优化的批量获取）
    Ok(tools)
  }

  /// 执行 LLM 调用，包含内存管理
  async fn execute_llm_with_memory(
    &self,
    _llm: &NodeSupplier,
    messages: &[Message],
    _config: &AiAgentConfig,
    memory_accessor: &Option<SimpleMemoryAccessor>,
  ) -> Result<SupplyResult, NodeExecutionError> {
    // 创建包含历史消息的输入数据
    debug!("Calling LLM with {} messages", messages.len());

    // 这里应该调用实际的 LLM 节点
    // 目前返回一个模拟的结果
    let mut mock_result = ahash::HashMap::default();
    mock_result.insert("content".to_string(), json!("This is a mock response from the LLM with memory integration."));
    mock_result.insert("role".to_string(), json!("assistant"));
    mock_result.insert("timestamp".to_string(), json!(now_offset().to_rfc3339()));

    // 如果有内存，记录助手响应（注意：在简化架构中不能直接保存）
    if let Some(memory) = memory_accessor
      && let Some(content) = mock_result.get("content")
      && let Some(content_str) = content.as_str()
    {
      debug!("Would save assistant response to memory session {}: {}", memory.session_id, content_str);
      // 在实际的工作流执行中，响应会在下一轮通过 SimpleMemoryNode 的输入自动保存
    }

    Ok(mock_result)
  }

  async fn execute_llm(
    &self,
    _llm: &NodeSupplier,
    _input_data: &ExecutionData,
    _config: &AiAgentConfig,
  ) -> Result<SupplyResult, NodeExecutionError> {
    // TODO: 实现 LLM 调用逻辑
    Err(NodeExecutionError::ConfigurationError("execute_llm not yet implemented".to_string()))
  }
}
