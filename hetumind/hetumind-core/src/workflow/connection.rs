use std::str::FromStr;

use serde::{Deserialize, Serialize};

use fusion_common::{
  DataError,
  helper::{default_bool_true, default_usize_0, is_true},
};
use typed_builder::TypedBuilder;

use super::NodeName;

/// 连接类型索引
pub type ConnectionIndex = usize;

/// 节点连接类型 - 使用枚举确保类型安全
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ConnectionKind {
  /// 传统工作流的主要数据流
  /// - 特点：最常用的连接类型，用于节点间传递业务数据
  /// - 示例：HTTP请求 → 数据处理 → 数据库写入
  Main = 1,

  /// 错误端口
  Error = 2,

  /// AI 工作流的主要数据流
  /// - 特点：用于复杂AI工作流的控制和协调
  /// - 示例：多步骤AI推理、决策链
  AiAgent = 101,

  /// 为AI代理提供可调用的工具
  /// - 特点：AI代理可以动态调用这些工具来完成任务
  /// - 示例：计算器工具、API调用工具、数据查询工具
  AiTool = 111,

  // AI 模型和解析连接
  AiLanguageModel = 121,
  AiOutputParser = 122,
  AiMemory = 123,

  // AI 数据处理连接
  /// 文档加载器
  AiDocument = 131,
  /// 向量嵌入
  AiEmbedding = 132,
  /// 向量检索器
  AiRetriever = 133,
  /// 向量存储
  AiVectorStore = 134,
  /// 重排序器
  AiReranker = 135,
  /// 文本分割器
  AiTextSplitter = 136,
}

impl FromStr for ConnectionKind {
  type Err = Box<dyn core::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    todo!()
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

/// 单个连接定义
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct Connection {
  /// 节点ID
  #[builder(setter(into))]
  node_name: NodeName,

  /// 端口类型
  kind: ConnectionKind,

  /// 端口索引
  #[serde(default = "default_usize_0")]
  index: ConnectionIndex,

  /// 连接条件 (可选，用于条件连接)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(strip_option))]
  condition: Option<ConnectionCondition>,

  /// 连接权重 (用于加权合并)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[builder(default, setter(strip_option))]
  weight: Option<i32>,

  /// 是否启用
  #[serde(default = "default_bool_true", skip_serializing_if = "is_true")]
  #[builder(default = true)]
  enabled: bool,
}

impl Connection {
  pub fn new(node_name: impl Into<NodeName>, kind: ConnectionKind, index: ConnectionIndex) -> Self {
    Self::builder().node_name(node_name).kind(kind).index(index).build()
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
