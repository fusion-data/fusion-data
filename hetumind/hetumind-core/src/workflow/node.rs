use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::types::IconColor;
use crate::version::Version;
use crate::workflow::{
  ExecutionDataMap, InputPortConfig, NodeExecutionContext, NodeExecutionError, NodeGroupKind, NodeProperty,
  OutputPortConfig,
};

/// The unique name of a node within a workflow. It is used to identify nodes configured in the workflow definition
/// and can be used to locate the node within the workflow.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::From, derive_more::Into)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
#[serde(transparent)]
pub struct NodeName(String);

impl NodeName {
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl AsRef<str> for NodeName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl Deref for NodeName {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<&str> for NodeName {
  fn from(id: &str) -> Self {
    Self(id.to_string())
  }
}

impl std::fmt::Display for NodeName {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

/// Node type, used to uniquely identify a node. Different versions of the same type of node use the same NodeKind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Into)]
#[serde(transparent)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
pub struct NodeKind(String);

impl NodeKind {
  pub fn new(kind: impl Into<String>) -> Self {
    Self(kind.into())
  }

  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl Deref for NodeKind {
  type Target = str;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl AsRef<str> for NodeKind {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<String> for NodeKind {
  fn from(id: String) -> Self {
    Self(id)
  }
}

impl From<&str> for NodeKind {
  fn from(id: &str) -> Self {
    Self(id.to_string())
  }
}

impl std::fmt::Display for NodeKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

/// Node 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
  /// 唯一标识一种类型的节点，可以存在多个不同的版本。PK
  pub kind: NodeKind,

  /// 版本号
  pub version: Version,

  /// 节点分组
  pub groups: Vec<NodeGroupKind>,

  /// 显示名称
  pub display_name: String,

  /// 节点描述
  pub description: Option<String>,

  /// 输入端口定义。与 node 的 outputs 对应。
  pub inputs: Vec<InputPortConfig>,

  /// 输出端口定义。
  pub outputs: Vec<OutputPortConfig>,

  /// 属性定义
  pub properties: Vec<NodeProperty>,

  /// 官方文档URL
  pub document_url: Option<String>,

  /// 子标题
  pub sub_title: Option<String>,

  /// 是否隐藏
  pub hidden: bool,

  /// 当个工作流中允许最多允许配置多少个此类型的节点，默认不限制
  pub max_nodes: Option<u32>,

  /// 节点图标。（支持 FontAwesome 图标或文件图标）或自定义图标URL
  pub icon: Option<String>,

  /// 图标颜色
  pub icon_color: Option<IconColor>,

  /// 自定义图标URL
  pub icon_url: Option<String>,

  /// 徽章图标URL
  pub badge_icon_url: Option<String>,
}

impl NodeDefinition {
  /// Create a new NodeDefinition with required fields
  pub fn new(kind: impl Into<NodeKind>, display_name: impl Into<String>) -> Self {
    Self {
      kind: kind.into(),
      version: Version::new(1, 0, 0),
      groups: Vec::new(),
      display_name: display_name.into(),
      description: None,
      inputs: Vec::new(),
      outputs: Vec::new(),
      properties: Vec::new(),
      document_url: None,
      sub_title: None,
      hidden: false,
      max_nodes: None,
      icon: None,
      icon_color: None,
      icon_url: None,
      badge_icon_url: None,
    }
  }

  pub fn with_version(mut self, version: impl Into<Version>) -> Self {
    self.version = version.into();
    self
  }

  // Methods for Option<T> fields
  pub fn with_description(mut self, description: impl Into<String>) -> Self {
    self.description = Some(description.into());
    self
  }

  pub fn with_document_url(mut self, document_url: impl Into<String>) -> Self {
    self.document_url = Some(document_url.into());
    self
  }

  pub fn with_sub_title(mut self, sub_title: impl Into<String>) -> Self {
    self.sub_title = Some(sub_title.into());
    self
  }

  pub fn with_max_nodes(mut self, max_nodes: u32) -> Self {
    self.max_nodes = Some(max_nodes);
    self
  }

  pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
    self.icon = Some(icon.into());
    self
  }

  pub fn with_icon_color(mut self, icon_color: impl Into<IconColor>) -> Self {
    self.icon_color = Some(icon_color.into());
    self
  }

  pub fn with_icon_url(mut self, icon_url: impl Into<String>) -> Self {
    self.icon_url = Some(icon_url.into());
    self
  }

  pub fn with_badge_icon_url(mut self, badge_icon_url: impl Into<String>) -> Self {
    self.badge_icon_url = Some(badge_icon_url.into());
    self
  }

  pub fn with_inputs<I>(mut self, inputs: I) -> Self
  where
    I: IntoIterator<Item = InputPortConfig>,
  {
    self.inputs = inputs.into_iter().collect();
    self
  }

  // Methods for Vec<T> fields
  pub fn add_input(mut self, input: InputPortConfig) -> Self {
    self.inputs.push(input);
    self
  }

  pub fn with_outputs<I>(mut self, outputs: I) -> Self
  where
    I: IntoIterator<Item = OutputPortConfig>,
  {
    self.outputs = outputs.into_iter().collect();
    self
  }

  pub fn add_output(mut self, output: OutputPortConfig) -> Self {
    self.outputs.push(output);
    self
  }

  pub fn with_properties<I>(mut self, properties: I) -> Self
  where
    I: IntoIterator<Item = NodeProperty>,
  {
    self.properties = properties.into_iter().collect();
    self
  }

  pub fn add_property(mut self, property: NodeProperty) -> Self {
    self.properties.push(property);
    self
  }

  pub fn with_groups<I>(mut self, groups: I) -> Self
  where
    I: IntoIterator<Item = NodeGroupKind>,
  {
    self.groups = groups.into_iter().collect();
    self
  }

  pub fn add_group(mut self, group: NodeGroupKind) -> Self {
    self.groups.push(group);
    self
  }
}

#[cfg(feature = "with-db")]
fusionsql::generate_string_newtype_to_sea_query_value!(Struct: NodeName, Struct: NodeKind);

pub trait Node {
  fn default_version(&self) -> &Version;

  fn node_executors(&self) -> &[NodeExecutor];

  fn kind(&self) -> NodeKind;

  fn versions(&self) -> Vec<Version> {
    self.node_executors().iter().map(|node| node.definition().version.clone()).collect()
  }

  fn get_node_executor(&self, version: &Version) -> Option<NodeExecutor> {
    self.node_executors().iter().find(|node| node.definition().version == *version).cloned()
  }

  fn default_node_executor(&self) -> Option<NodeExecutor> {
    self.get_node_executor(self.default_version())
  }
}

#[async_trait]
pub trait NodeExecutable {
  /// Initialize the node. This can be used to implement node initialization logic, such as loading configuration, initializing resources, etc.
  async fn init(&mut self, _context: &NodeExecutionContext) -> Result<(), NodeExecutionError> {
    Ok(())
  }

  /// Execute the node
  ///
  /// Returns:
  /// - On success, returns data for multiple output ports, with the first output port starting from 0
  /// - On failure, returns an error
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError>;

  /// Get Node definition
  fn definition(&self) -> Arc<NodeDefinition>;
}

/// Node Executor Type
pub type NodeExecutor = Arc<dyn NodeExecutable + Send + Sync>;
