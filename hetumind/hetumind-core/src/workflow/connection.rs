use fusion_common::helper::{default_bool_true, default_usize_0, is_true};
use serde::{Deserialize, Serialize};

use super::NodeName;

/// 连接类型索引
pub type ConnectionIndex = usize;

/// 节点连接类型 - 使用枚举确保类型安全
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  strum::Display,
  strum::AsRefStr,
  strum::EnumString,
  strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ConnectionKind {
  /// 传统工作流的主要数据流
  /// - 特点：最常用的连接类型，用于节点间传递业务数据
  /// - 示例：HTTP请求 → 数据处理 → 数据库写入
  Main,

  /// 错误端口
  Error,

  /// AI 工作流的主要数据流
  /// - 特点：用于复杂AI工作流的控制和协调
  /// - 示例：多步骤AI推理、决策链
  AiAgent,

  /// 为AI代理提供可调用的工具
  /// - 特点：AI代理可以动态调用这些工具来完成任务
  /// - 示例：计算器工具、API调用工具、数据查询工具
  AiTool,

  // AI 模型和解析连接
  /// Large Language Model
  AiLM,
  /// 输出解析器
  AiOutputParser,
  /// 记忆模块，用于存储和检索AI代理的记忆
  AiMemory,

  // AI 数据处理连接
  /// 文档加载器
  AiDocument,
  /// 向量嵌入
  AiEmbedding,
  /// 向量检索器
  AiRetriever,
  /// 向量存储
  AiVectorStore,
  /// 重排序器
  AiReranker,
  /// 文本分割器
  AiTextSplitter,
}

impl ConnectionKind {
  pub fn can_used_workflow(&self) -> bool {
    matches!(self, ConnectionKind::Main | ConnectionKind::Error)
  }
}

/// 连接条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCondition {
  /// 条件表达式
  pub expression: String,
  /// 条件描述
  pub description: Option<String>,
}

impl ConnectionCondition {
  pub fn new(expression: impl Into<String>) -> Self {
    Self { expression: expression.into(), description: None }
  }

  pub fn with_expression(mut self, expression: impl Into<String>) -> Self {
    self.expression = expression.into();
    self
  }

  pub fn with_description(mut self, description: impl Into<String>) -> Self {
    self.description = Some(description.into());
    self
  }
}

/// 单个连接定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
  /// 节点ID
  pub node_name: NodeName,

  /// 端口类型
  pub kind: ConnectionKind,

  /// 端口索引
  #[serde(default = "default_usize_0")]
  pub index: ConnectionIndex,

  /// 连接条件 (可选，用于条件连接)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub condition: Option<ConnectionCondition>,

  /// 连接权重 (用于加权合并)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub weight: Option<i32>,

  /// 是否启用
  #[serde(default = "default_bool_true", skip_serializing_if = "is_true")]
  pub enabled: bool,
}

impl Connection {
  pub fn new(node_name: impl Into<NodeName>, kind: ConnectionKind, index: ConnectionIndex) -> Self {
    Self { node_name: node_name.into(), kind, index, condition: None, weight: None, enabled: true }
  }

  pub fn with_condition(mut self, condition: ConnectionCondition) -> Self {
    self.condition = Some(condition);
    self
  }

  pub fn with_weight(mut self, weight: i32) -> Self {
    self.weight = Some(weight);
    self
  }

  pub fn with_enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  pub fn node_name(&self) -> &NodeName {
    &self.node_name
  }

  pub fn kind(&self) -> ConnectionKind {
    self.kind
  }

  pub fn index(&self) -> ConnectionIndex {
    self.index
  }

  pub fn condition(&self) -> Option<&ConnectionCondition> {
    self.condition.as_ref()
  }

  pub fn weight(&self) -> Option<i32> {
    self.weight
  }

  pub fn enabled(&self) -> bool {
    self.enabled
  }
}
