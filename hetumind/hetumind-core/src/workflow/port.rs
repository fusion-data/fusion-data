use fusion_common::helper::{default_bool_true, default_u32_1};
use serde::{Deserialize, Serialize};

use super::{NodeConnectionKind, NodeName};

/// 节点输入过滤器
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortInputFilter {
  includes: Vec<NodeName>,
  excludes: Vec<NodeName>,
}

impl Default for PortInputFilter {
  fn default() -> Self {
    Self::new()
  }
}

impl PortInputFilter {
  pub fn new() -> Self {
    Self { includes: Vec::default(), excludes: Vec::default() }
  }

  pub fn with_includes<I, V>(mut self, includes: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<NodeName>,
  {
    self.includes = includes.into_iter().map(|v| v.into()).collect();
    self
  }

  pub fn add_include(mut self, include: impl Into<NodeName>) -> Self {
    self.includes.push(include.into());
    self
  }

  pub fn with_excludes<I, V>(mut self, excludes: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<NodeName>,
  {
    self.excludes = excludes.into_iter().map(|v| v.into()).collect();
    self
  }

  pub fn add_exclude(mut self, exclude: impl Into<NodeName>) -> Self {
    self.excludes.push(exclude.into());
    self
  }
}

/// 节点输入连接
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputPortConfig {
  pub kind: NodeConnectionKind,

  /// 显示名称。在UI渲染时，系统会根据以下逻辑确定显示的标签
  pub display_name: String,

  #[serde(default = "default_bool_true")]
  pub required: bool,

  pub filter: Option<PortInputFilter>,

  #[serde(default = "default_u32_1")]
  pub max_connections: u32,

  pub category: Option<String>,
}

impl InputPortConfig {
  pub fn new(kind: NodeConnectionKind, display_name: impl Into<String>) -> Self {
    Self { kind, display_name: display_name.into(), required: true, filter: None, max_connections: 1, category: None }
  }

  pub fn with_kind(mut self, kind: NodeConnectionKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
    self.display_name = display_name.into();
    self
  }

  pub fn with_required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }

  pub fn with_filter(mut self, filter: impl Into<PortInputFilter>) -> Self {
    self.filter = Some(filter.into());
    self
  }

  pub fn with_max_connections(mut self, max_connections: u32) -> Self {
    self.max_connections = max_connections;
    self
  }

  pub fn with_category(mut self, category: impl Into<String>) -> Self {
    self.category = Some(category.into());
    self
  }
}

/// 节点输出配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPortConfig {
  pub kind: NodeConnectionKind,

  /// 显示名称。在UI渲染时，系统会根据以下逻辑确定显示的标签
  pub display_name: String,

  #[serde(default = "default_bool_true")]
  pub required: bool,

  #[serde(default = "default_u32_1")]
  pub max_connections: u32,

  pub category: Option<String>,
}

impl OutputPortConfig {
  pub fn new(kind: NodeConnectionKind, display_name: impl Into<String>) -> Self {
    Self { kind, display_name: display_name.into(), required: true, max_connections: 1, category: None }
  }

  pub fn with_kind(mut self, kind: NodeConnectionKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
    self.display_name = display_name.into();
    self
  }

  pub fn with_required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }

  pub fn with_max_connections(mut self, max_connections: u32) -> Self {
    self.max_connections = max_connections;
    self
  }

  pub fn with_category(mut self, category: impl Into<String>) -> Self {
    self.category = Some(category.into());
    self
  }
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
