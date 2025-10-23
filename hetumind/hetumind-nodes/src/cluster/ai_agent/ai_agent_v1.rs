use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use async_trait::async_trait;
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, ExecutionDataItems, ExecutionDataMap, NodeDefinition, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeSupplier, RegistrationError, SupplyResult, make_execution_data_map,
};
use mea::rwlock::RwLock;
use rig::message::Message;
use rig::{client::CompletionClient, completion::Prompt};
use serde_json::json;
use uuid::Uuid;

use crate::cluster::ai_agent::parameters::ToolExecutionStatus;
use crate::cluster::ai_agent::tool_manager::ToolManager;
use crate::cluster::ai_agent::utils::create_base_definition;

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

    // 2. 获取 memory （如果有的话）
    // TODO: 实现 memory 获取逻辑

    // 3. 获取连接的工具
    let _tools = self.get_tools(context).await?;

    // 4. 获取连接的 LLM 实例
    let agent = self.get_llm_instance(context).await?;

    // 5. 执行 Agent
    // 将 input_data merge config + memory + tools 传入 agent 执行
    let result = //if config.enable_streaming() {
      // self.execute_agent_streaming(&agent, &input_data, &config).await?
    // } else {
      self.execute_llm(&agent, &input_data, &config).await?;
    // };

    // 6. 返回最终结果
    let mut data_map = ExecutionDataMap::default();
    data_map.insert(
      ConnectionKind::Main,
      vec![ExecutionDataItems::new_item(ExecutionData::new_json(
        json!({
            "response": result,
            "node_kind": &self.definition().kind,
            "streaming": config.enable_streaming,
            "timestamp": now_offset(),
        }),
        Some(DataSource::new(context.current_node_name().clone(), ConnectionKind::Main, 0)),
      ))],
    );
    Ok(data_map)
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

  async fn get_tools(&self, _context: &NodeExecutionContext) -> Result<Vec<JsonValue>, NodeExecutionError> {
    let tools = Vec::new();
    // TODO 获取所有连接的工具（使用优化的批量获取）
    Ok(tools)
  }

  async fn execute_llm(
    &self,
    llm: &NodeSupplier,
    input_data: &ExecutionData,
    _config: &AiAgentConfig,
  ) -> Result<SupplyResult, NodeExecutionError> {
    let prompt = input_data
      .get_value::<Message>("prompt")
      .map_err(|e| NodeExecutionError::invalid_input("Get paremeter 'prompt' failed."))?;

    // llm.supply(context).await
    todo!()
  }
}
