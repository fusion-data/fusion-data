//! DeepSeek V1 最小端到端驱动示例
//!
//! 运行前请确保环境变量 `DEEPSEEK_API_KEY` 已设置。
//! 运行命令：
//!   cargo run -p hetumind-nodes --example deepseek_smoke
//!
//! 本示例构建一个最小的 Workflow/NodeExecutionContext，并调用 DeepseekV1 节点的 execute 方法，
//! 以验证系统提示、历史消息（用户侧）和参数绑定（temperature/max_tokens）是否生效。

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::binary_storage::BinaryDataManager;
use hetumind_core::binary_storage::{BinaryDataMetadata, BinaryDataStorage, BinaryStorageError};
use hetumind_core::types::JsonValue;
use hetumind_core::workflow::Node; // 引入 trait 以访问 node_executors
use hetumind_core::workflow::{
  ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, ExecutionId, NodeElement, NodeExecutionContext,
  NodeKind, NodeName, NodeRegistry, Workflow, WorkflowId,
};
use hetumind_nodes::llm::deepseek_node::DeepseekModelNode;
use serde_json::json;
use uuid::Uuid;

/// 简单内存存储（用于 BinaryDataManager 的最小可用实现）
struct SimpleMemoryStorage;

#[async_trait]
impl BinaryDataStorage for SimpleMemoryStorage {
  async fn store(&self, _data: Vec<u8>, _metadata: &BinaryDataMetadata) -> Result<String, BinaryStorageError> {
    Ok(format!("mem_{}", uuid::Uuid::now_v7()))
  }
  async fn retrieve(&self, _key: &str) -> Result<Vec<u8>, BinaryStorageError> {
    Ok(vec![])
  }
  async fn get_metadata(&self, _key: &str) -> Result<BinaryDataMetadata, BinaryStorageError> {
    Ok(BinaryDataMetadata::default())
  }
  async fn delete(&self, _key: &str) -> Result<(), BinaryStorageError> {
    Ok(())
  }
  async fn exists(&self, _key: &str) -> Result<bool, BinaryStorageError> {
    Ok(false)
  }
  async fn list(&self, _prefix: &str) -> Result<Vec<String>, BinaryStorageError> {
    Ok(vec![])
  }
  fn storage_type_name(&self) -> &'static str {
    "memory"
  }
}

/// 构造最小输入数据（包含系统提示、历史消息与参数）
fn build_input_json() -> JsonValue {
  json!({
    "system_prompt": "你是一个 helpful assistant。请简洁回答。",
    "messages": [
      {"role": "user", "content": "请简要介绍 Rust 的所有权模型。"},
      {"role": "assistant", "content": "好的，我将简要介绍。"},
      {"role": "user", "content": "并比较与 GC 的差异。"}
    ],
    "temperature": 0.6,
    "max_tokens": 512
  })
}

/// 构造最小节点参数（DeepseekV1 节点配置）
fn build_node_parameters() -> hetumind_core::workflow::ParameterMap {
  let mut map = serde_json::Map::new();
  map.insert("model".to_string(), json!("deepseek-chat"));
  map.insert("max_tokens".to_string(), json!(1024));
  map.insert("temperature".to_string(), json!(0.6));
  map.insert("top_p".to_string(), json!(1));
  map.insert("stop_sequences".to_string(), json!(null));
  // 通过环境变量解析 API Key（需提前 export DEEPSEEK_API_KEY）
  map.insert(
    "common".to_string(),
    json!({
      "api_key": "${env:DEEPSEEK_API_KEY}",
      "max_tokens": 1024,
      "temperature": 0.6,
      "top_p": 1.0,
      "stream": false,
      "timeout": 60
    }),
  );
  hetumind_core::workflow::ParameterMap::new(map)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv().ok();
  // 1) 构建最小 Workflow 与 Node
  let workflow_id = WorkflowId::from(Uuid::now_v7());
  let mut workflow = Workflow::new(workflow_id, "deepseek_smoke");

  let node_kind = NodeKind::new(hetumind_nodes::constants::DEEPSEEK_MODEL_NODE_KIND);
  let node_name = NodeName::from("deepseek_v1");
  let node = NodeElement::new(node_kind.clone(), node_name.clone()).with_parameters(build_node_parameters());
  workflow.nodes.push(node);

  // 2) 构造输入数据（AiLM 端口）
  let input_json = build_input_json();
  let input_data = ExecutionData::new_json(input_json, None);
  let mut parents_results: ExecutionDataMap = ExecutionDataMap::default();
  parents_results.insert(ConnectionKind::AiLM, vec![ExecutionDataItems::new_items(vec![input_data])]);

  // 3) 构造 NodeExecutionContext（最小二进制存储 + 空注册表）
  let storage = Arc::new(SimpleMemoryStorage);
  let binary_data_manager = BinaryDataManager::with_default_cache(storage)?;
  let node_registry = NodeRegistry::new();
  let exec_id = ExecutionId::from(Uuid::now_v7());
  let ctx = NodeExecutionContext::new(
    exec_id,
    Arc::new(workflow),
    node_name.clone(),
    parents_results,
    binary_data_manager,
    node_registry,
  );

  // 4) 执行 DeepseekV1 节点
  let model_node = DeepseekModelNode::new().expect("create deepseek model node");
  let executor = model_node.node_executors().first().expect("deepseek v1 executor").clone();
  let outputs = executor.execute(&ctx).await?;

  // 5) 打印输出（AiLM 端口）
  if let Some(items) = outputs.get(&ConnectionKind::AiLM) {
    if let Some(ExecutionDataItems::Items(vec)) = items.first() {
      for data in vec {
        println!("LLM Output JSON: {}", serde_json::to_string_pretty(data.json())?);
      }
    }
  } else {
    println!("No output on AiLM port");
  }

  Ok(())
}
