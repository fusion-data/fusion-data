use fusion_common::helper::{default_bool_true, default_u32_1};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{ConnectionKind, NodeName};

/// 节点输入过滤器
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortInputFilter {
  includes: Vec<NodeName>,
  excludes: Vec<NodeName>,
}

/// 节点输入连接
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
pub struct InputPortConfig {
  #[builder(setter(into))]
  pub kind: ConnectionKind,

  /// 显示名称。在UI渲染时，系统会根据以下逻辑确定显示的标签
  #[builder(setter(into))]
  pub display_name: String,

  #[serde(default = "default_bool_true")]
  #[builder(default = true)]
  pub required: bool,

  #[builder(default, setter(strip_option))]
  pub filter: Option<PortInputFilter>,

  #[serde(default = "default_u32_1")]
  #[builder(default = 1)]
  pub max_connections: u32,

  #[builder(default, setter(into, strip_option))]
  pub category: Option<String>,
}

/// 节点输出配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
pub struct OutputPortConfig {
  pub kind: ConnectionKind,

  /// 显示名称。在UI渲染时，系统会根据以下逻辑确定显示的标签
  #[builder(setter(into))]
  pub display_name: String,

  #[serde(default = "default_bool_true")]
  #[builder(default = true)]
  pub required: bool,

  #[serde(default = "default_u32_1")]
  #[builder(default = 1)]
  pub max_connections: u32,

  #[builder(default, setter(into, strip_option))]
  pub category: Option<String>,
}

// /// 节点输出端口配置
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct OutputPort {
//   /// 端口类型
//   kind: ConnectionKind,
//   /// 端口索引
//   index: PortIndex,
//   /// 连接的目标列表，从 connections 生成
//   targets: Vec<ConnectionPort>,
// }

// impl OutputPort {
//   pub fn new(kind: ConnectionKind, index: PortIndex, targets: Vec<ConnectionPort>) -> Self {
//     Self { kind, index, targets }
//   }

//   pub fn display_name(&self) -> String {
//     format!("{:?}:{}", self.kind, self.index)
//   }

//   pub fn kind(&self) -> ConnectionKind {
//     self.kind
//   }

//   pub fn index(&self) -> PortIndex {
//     self.index
//   }

//   pub fn targets(&self) -> &[ConnectionPort] {
//     &self.targets
//   }
// }

// /// 节点输入端口配置
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct InputPort {
//   /// 端口类型
//   kind: ConnectionKind,
//   /// 端口索引
//   index: PortIndex,
//   /// 是否必需
//   required: bool,
//   /// 连接的源端点，从 connections 生成
//   source: Option<ConnectionPort>,
// }

// impl InputPort {
//   pub fn new(kind: ConnectionKind, index: PortIndex, required: bool, source: Option<ConnectionPort>) -> Self {
//     Self { kind, index, required, source }
//   }

//   pub fn display_name(&self) -> String {
//     format!("{:?}:{}", self.kind, self.index)
//   }

//   pub fn kind(&self) -> ConnectionKind {
//     self.kind
//   }

//   pub fn index(&self) -> PortIndex {
//     self.index
//   }

//   pub fn required(&self) -> bool {
//     self.required
//   }

//   pub fn source(&self) -> Option<&ConnectionPort> {
//     self.source.as_ref()
//   }
// }
