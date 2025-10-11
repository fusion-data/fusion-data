//! 节点执行
use std::sync::Arc;

use fusion_common::ahash::HashMap;
use fusion_common::time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
  expression::ExpressionEvaluator,
  types::{BinaryFileKind, JsonValue},
  user::UserId,
  workflow::{ExecutionId, GetNodeParameterOptions, ValidationError, Workflow},
};

use super::{ConnectionIndex, ConnectionKind, NodeExecutionError, NodeExecutionStatus, NodeName, WorkflowNode};

#[derive(Debug, TypedBuilder)]
pub struct NodeExecutionContext {
  /// 执行ID
  pub execution_id: ExecutionId,
  /// 工作流引用
  pub workflow: Arc<Workflow>,
  /// 当前节点ID
  pub current_node_name: NodeName,
  /// 输入数据
  pub input_data: ExecutionDataMap,
  /// 执行开始时间
  pub started_at: OffsetDateTime,
  /// 用户ID
  pub user_id: Option<UserId>,
  /// 环境变量
  pub env_vars: HashMap<String, String>,

  pub expression_evaluator: ExpressionEvaluator,
}

impl NodeExecutionContext {
  pub fn current_node(&self) -> Result<&WorkflowNode, NodeExecutionError> {
    match self.workflow.nodes.iter().find(|n| n.name == self.current_node_name) {
      Some(node) => Ok(node),
      None => Err(NodeExecutionError::NodeNotFound {
        workflow_id: self.workflow.id.clone(),
        node_name: self.current_node_name.clone(),
      }),
    }
  }

  pub fn get_input_items(
    &self,
    connection_kind: ConnectionKind,
    input_index: ConnectionIndex,
  ) -> Option<ExecutionDataItems> {
    let input_items = self.input_data.get(&connection_kind)?;

    if input_index >= input_items.len() {
      return None;
    }

    Some(input_items[input_index].clone())
  }

  pub fn get_node_parameter(
    &self,
    parameter_name: impl AsRef<str>,
    default_value: Option<impl Into<JsonValue>>,
    options: Option<GetNodeParameterOptions>,
  ) -> Result<JsonValue, NodeExecutionError> {
    let node = self.current_node()?;

    let value: JsonValue = node
      .get_optional_parameter(parameter_name.as_ref())
      .or_else(|| default_value.map(|v| v.into()))
      .ok_or_else(|| ValidationError::required_field_missing(parameter_name.as_ref()))?;

    if options.is_some_and(|o| o.is_raw_expressions()) {
      return Ok(value);
    }

    todo!()
  }
}

/// 节点之间的执行数据（传递），是工作流中流动的基本数据单元。
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionDataInner {
  /// 数据内容
  pub json: JsonValue,
  /// 二进制数据引用
  pub binary: Option<BinaryDataReference>,
  /// 来源信息。保留了数据项在原始批次中的索引。在循环、合并等操作中，这个索引可以用来保持数据的对应关系。
  pub source: Option<DataSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionData(Arc<ExecutionDataInner>);

impl ExecutionData {
  pub fn new_json(json: JsonValue, source: Option<DataSource>) -> Self {
    Self(Arc::new(ExecutionDataInner { json, binary: None, source }))
  }

  pub fn new_binary(binary: BinaryDataReference, source: Option<DataSource>) -> Self {
    Self(Arc::new(ExecutionDataInner { json: JsonValue::Null, binary: Some(binary), source }))
  }

  pub fn json(&self) -> &JsonValue {
    &self.0.json
  }

  pub fn binary(&self) -> Option<&BinaryDataReference> {
    self.0.binary.as_ref()
  }

  pub fn source(&self) -> Option<&DataSource> {
    self.0.source.as_ref()
  }
}

/// 执行数据来源
///
/// - 实现了“数据血缘”追踪，让每一条数据都携带着它的来源信息。这对于调试复杂的工作流至关重要，
///   我们可以清晰地知道一个数据项是哪个节点的哪个端口产生的。
/// - 优势: 显式地携带 `DataSource` 让数据流变得可追溯。我们可以轻松构建一个工具来可视化任何一个
///   `ExecutionData` 的完整生命周期。这降低了系统的“魔术性”，提高了透明度和可维护性。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
  /// 来源节点ID
  pub node_name: NodeName,
  /// 来源输出端口
  pub output_port: ConnectionKind,
  /// 数据索引
  pub output_index: ConnectionIndex,
}

/// 二进制数据引用
///
/// - 引用: `file_key` 是二进制数据在对象存储中的唯一标识符，（比如 S3、本地文件存储），我们只在节点间传递这个轻量级的引用。
/// - 优势: 您的引用设计非常出色，尤其适合云原生和高性能场景。
///   - 内存效率: 避免了在内存中复制和持有大量二进制数据（如视频、大文件），极大地降低了执行引擎的内存占用。
///   - 可扩展性: 在分布式或无服务器（Lambda）环境中，无法在两个独立的函数实例之间直接传递内存中的 Buffer。而传递一个 file_id (例如 S3 object key) 是简单而高效的。这个设计为 hetumind-lambda 的实现铺平了道路。
///   - 持久化: 它强制将二进制数据的处理流程规范化：“接收 -> 存到对象存储 -> 获取引用 ID -> 传递引用”。这使得工作流的中间状态也更容易持久化和恢复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDataReference {
  /// 文件对象key
  pub file_key: String,
  /// MIME 类型
  pub mime_kind: String,
  /// 文件大小
  pub file_size: u64,
  /// 文件名
  pub file_name: Option<String>,
  /// 文件类型
  pub file_kind: Option<BinaryFileKind>,
  /// 文件扩展名
  pub file_extension: Option<String>,
  /// 文件目录
  pub directory: Option<String>,
}

/// 多个节点的执行数据。用于聚合所有节点的执行结果，或单个节点的所有输入结果（它可能同时收到来自多个父节点的输入数据）。
pub type NodesExecutionMap = HashMap<NodeName, ExecutionDataMap>;

/// 单个节点的所有 **输入/输出** 连接的数据。 (key: 连接类型, value: 多个连接的数据)
/// 相同类型（[ConnectionKind]）的多个连接数据，以数组形式存储。
pub type ExecutionDataMap = HashMap<ConnectionKind, Vec<ExecutionDataItems>>;

/// 一个连接的 **输出/输入** 数据
pub type VecExecutionData = Vec<ExecutionData>;

/// 一个连接的数据（可表达为空）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecutionDataItems {
  /// 空数据，用于数据流中没有数据的情况。比如：此连接的数据还未达到（input），或者此连接未使用（output）。
  Null,
  /// 数据数组，用于数据流中有多条数据的情况。比如：此连接的数据已达到（input），或者此连接的数据已填充（output）。
  Items(VecExecutionData),
}

impl ExecutionDataItems {
  pub fn new_null() -> Self {
    Self::Null
  }

  pub fn new_items(items: VecExecutionData) -> Self {
    Self::Items(items)
  }

  pub fn get_data_items(&self) -> Option<VecExecutionData> {
    match self {
      ExecutionDataItems::Null => None,
      ExecutionDataItems::Items(items) => Some(items.clone()),
    }
  }

  pub fn len(&self) -> usize {
    match self {
      ExecutionDataItems::Null => 0,
      ExecutionDataItems::Items(items) => items.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      ExecutionDataItems::Null => true,
      ExecutionDataItems::Items(items) => items.is_empty(),
    }
  }
}

#[derive(Debug, TypedBuilder)]
pub struct NodeExecutionResult {
  /// 节点ID
  pub node_name: NodeName,
  /// 执行状态
  pub status: NodeExecutionStatus,
  /// 输出数据
  pub output_data: ExecutionDataMap, // Vec<ExecutionDataItems>,
  /// 错误信息
  #[builder(default, setter(strip_option))]
  pub error: Option<String>,
  /// 执行时长
  pub duration_ms: u64,
}

/// 一次节点执行的完整记录，默认保存在内存中。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecution {
  /// 节点执行唯一标识符
  pub id: Uuid,
  /// 所属执行ID
  pub execution_id: ExecutionId,
  /// 执行的节点ID
  pub node_name: NodeName,
  /// 执行状态
  pub status: NodeExecutionStatus,
  /// 开始时间
  pub started_at: OffsetDateTime,
  /// 结束时间
  pub finished_at: Option<OffsetDateTime>,
  /// 输入数据
  pub input_data: Option<serde_json::Value>,
  /// 输出数据
  pub output_data: Option<serde_json::Value>,
  /// 错误信息
  pub error: Option<String>,
  /// 重试次数
  pub retry_count: i32,
  /// 执行时长（毫秒）
  pub duration_ms: Option<u64>,
}
