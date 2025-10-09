use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use derive_builder::Builder;
use fusion_common::helper::default_bool_false;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::types::{IconColor, JsonValue};
use crate::version::Version;
use crate::workflow::{
  AssignmentKindOptions, ButtonConfig, CalloutAction, CodeAutocompleteType, CredentialKind, DataPathRequirement,
  DisplayOptions, EditorType, ExecutionDataMap, FieldType, FilterTypeOptions, InputPortConfig, LoadOptions,
  NodeExecutionContext, NodeExecutionError, NodeGroupKind, OutputPortConfig, ResourceMapperTypeOptions, SqlDialect,
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

/// 节点类型，用于唯一标识一个节点，相同类型的不同版本节点使用相同的 NodeKind
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Into)]
#[serde(transparent)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
pub struct NodeKind(String);

impl NodeKind {
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

/// 节点属性类型（元数据）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodePropertyKind {
  AssignmentCollection,
  Boolean,
  Button,
  Callout,
  Collection,
  Color,
  Credentials,
  CredentialsSelect,
  CurlImport,
  DateTime,
  Filter,
  FixedCollection,
  Hidden,
  Json,
  MultiOptions,
  Notice,
  Number,
  Options,
  ResourceLocator,
  ResourceMapper,
  #[default]
  String,
  WorkflowSelector,
}

// 节点属性类型选项 - 主结构体
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct NodePropertyKindOptions {
  /// 按钮配置 (支持: [NodePropertyKind::Button])
  #[builder(default, setter(strip_option))]
  pub button_config: Option<ButtonConfig>,

  /// 容器类名 (支持: [NodePropertyKind::Notice])
  #[builder(default, setter(strip_option))]
  pub container_class: Option<String>,

  /// 始终打开编辑窗口 (支持: [NodePropertyKind::Json])
  #[builder(default, setter(strip_option))]
  pub always_open_edit_window: Option<bool>,

  /// 代码自动完成 (支持: [NodePropertyKind::String])
  #[builder(default, setter(strip_option))]
  pub code_autocomplete: Option<CodeAutocompleteType>,

  /// 编辑器类型 (支持: [NodePropertyKind::String])
  #[builder(default, setter(strip_option))]
  pub editor: Option<EditorType>,

  /// 编辑器只读 (支持: [NodePropertyKind::String])
  #[builder(default, setter(strip_option))]
  pub editor_is_read_only: Option<bool>,

  /// SQL 方言 (支持: [EditorType::SqlEditor])
  #[builder(default, setter(strip_option))]
  pub sql_dialect: Option<SqlDialect>,

  /// 加载选项依赖 (支持: [NodePropertyKind::Options])
  #[builder(default, setter(into, strip_option))]
  pub load_options_depends_on: Option<Vec<String>>,

  /// 加载选项方法 (支持: [NodePropertyKind::Options])
  #[builder(default, setter(strip_option))]
  pub load_options_method: Option<String>,

  /// 加载选项 (支持: [NodePropertyKind::Options])
  #[builder(default, setter(strip_option))]
  pub load_options: Option<LoadOptions>,

  /// 最大值 (支持: [NodePropertyKind::Number])
  #[builder(default, setter(strip_option))]
  pub max_value: Option<f64>,

  /// 最小值 (支持: [NodePropertyKind::Number])
  #[builder(default, setter(strip_option))]
  pub min_value: Option<f64>,

  /// 多个值 (支持: all [NodePropertyKind])
  #[builder(default, setter(strip_option))]
  pub multiple_values: Option<bool>,

  /// 多值按钮文本 (当 [Self::multiple_values]=true 时支持)
  #[builder(default, setter(into, strip_option))]
  pub multiple_value_button_text: Option<String>,

  /// 数字精度 (支持: [NodePropertyKind::Number])
  #[builder(default, setter(strip_option))]
  pub number_precision: Option<i32>,

  /// 密码字段 (支持: [NodePropertyKind::String])
  #[builder(default, setter(strip_option))]
  pub password: Option<bool>,

  /// 行数 (支持: [NodePropertyKind::String])
  #[builder(default, setter(into, strip_option))]
  pub rows: Option<i32>,

  /// 显示透明度 (支持: [NodePropertyKind::Color])
  #[builder(default, setter(strip_option))]
  pub show_alpha: Option<bool>,

  /// 可排序 (当 [Self::multiple_values]=true 时支持)
  #[builder(default, setter(strip_option))]
  pub sortable: Option<bool>,

  /// 可过期 (支持: [NodePropertyKind::Hidden]，仅在凭据中)
  #[builder(default, setter(strip_option))]
  pub expirable: Option<bool>,

  /// 资源映射器配置
  #[builder(default, setter(strip_option))]
  pub resource_mapper: Option<ResourceMapperTypeOptions>,

  /// 过滤器配置
  #[builder(default, setter(strip_option))]
  pub filter: Option<FilterTypeOptions>,

  /// 赋值配置
  #[builder(default, setter(strip_option))]
  pub assignment: Option<AssignmentKindOptions>,

  /// 最少必需字段数 (支持: [NodePropertyKind::FixedCollection])
  #[builder(default, setter(strip_option))]
  pub min_required_fields: Option<i32>,

  /// 最多允许字段数 (支持: [NodePropertyKind::FixedCollection])
  #[builder(default, setter(strip_option))]
  pub max_allowed_fields: Option<i32>,

  /// 调用动作 (支持: [NodePropertyKind::Callout])
  #[builder(default, setter(strip_option))]
  pub callout_action: Option<CalloutAction>,

  /// 其他扩展字段
  #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
  #[builder(default)]
  pub additional_properties: serde_json::Map<String, JsonValue>,
}

// 值提取器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePropertyValueExtractor {
  pub kind: String,
  pub regex: Option<String>,
  pub property: Option<String>,
}

// 节点属性模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePropertyMode {
  pub display_name: String,
  pub name: String,
  pub kind: NodePropertyKind,
}

/// 路由配置 - 简化版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePropertyRouting {
  pub request: Option<serde_json::Value>,
  pub output: Option<serde_json::Value>,
  pub operations: Option<serde_json::Value>,
}

/// 节点属性定义（元数据）
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
pub struct NodeProperty {
  /// 显示名称
  #[builder(setter(into))]
  pub display_name: String,

  /// 在节点定义中唯一标识一个属性。实际工作流使用时，由工作流节点的 parameters 中的 key 表示。
  #[builder(setter(into))]
  pub name: String,

  /// 属性类型。UI 渲染方式依赖于此
  #[builder(default)]
  pub kind: NodePropertyKind,

  /// 针对特定类型的附加配置选项
  #[builder(default, setter(strip_option))]
  pub kind_options: Option<NodePropertyKindOptions>,

  /// 是否必填
  #[builder(default = true)]
  pub required: bool,

  /// 参数的默认值
  #[builder(default, setter(strip_option))]
  pub value: Option<JsonValue>,

  /// 参数的描述信息
  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,

  /// 参数的提示信息
  #[builder(default, setter(into, strip_option))]
  pub hint: Option<String>,

  /// 禁用选项
  #[builder(default, setter(strip_option))]
  pub disable_options: Option<DisplayOptions>,

  /// 显示选项
  #[builder(default, setter(strip_option))]
  pub display_options: Option<DisplayOptions>,

  /// 选项
  #[builder(default, setter(strip_option))]
  pub options: Option<Vec<Box<NodeProperty>>>,

  /// 输入框占位符文本
  #[builder(default, setter(into, strip_option))]
  pub placeholder: Option<String>,

  /// 是否为节点级别设置
  #[builder(default, setter(strip_option))]
  pub is_node_setting: Option<bool>,

  /// 是否禁止使用数据表达式
  #[builder(default, setter(strip_option))]
  pub no_data_expression: Option<bool>,

  /// API 路由配置
  #[builder(default, setter(strip_option))]
  pub routing: Option<NodePropertyRouting>,

  /// 支持的凭据类型列表
  #[builder(default, setter(strip_option))]
  pub credential_kinds: Option<Vec<CredentialKind>>,

  /// 值提取配置
  #[builder(default, setter(strip_option))]
  pub extract_value: Option<NodePropertyValueExtractor>,

  /// 参数的不同模式配置
  #[builder(default, setter(strip_option))]
  pub modes: Option<Vec<NodePropertyMode>>,

  /// 需要的数据路径类型
  #[builder(default, setter(strip_option))]
  pub requires_data_path: Option<DataPathRequirement>,

  /// 是否不从父级继承此参数
  #[builder(default, setter(strip_option))]
  pub do_not_inherit: Option<bool>,

  /// 用于验证和类型转换的预期类型
  #[builder(default, setter(strip_option))]
  pub validate_type: Option<FieldType>,

  /// 执行期间是否跳过验证
  #[builder(default, setter(strip_option))]
  pub ignore_validation_during_execution: Option<bool>,

  /// 扩展属性
  #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
  #[builder(default)]
  additional_properties: serde_json::Map<String, JsonValue>,
}

impl NodeProperty {
  pub fn new_option(
    display_name: impl Into<String>,
    name: impl Into<String>,
    value: JsonValue,
    kind: NodePropertyKind,
  ) -> Self {
    Self::builder().display_name(display_name).name(name).value(value).kind(kind).build()
  }
}

/// Node 定义
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NodeDefinition {
  /// 唯一标识一种类型的节点，可以存在多个不同的版本。PK
  #[builder(setter(into))]
  pub kind: NodeKind,

  /// 版本号
  #[builder(setter(into))]
  pub version: Version,

  /// 节点分组
  #[builder(setter(into))]
  pub groups: Vec<NodeGroupKind>,

  /// 显示名称
  #[builder(setter(into))]
  pub display_name: String,

  /// 节点描述
  #[builder(default, setter(into, strip_option))]
  pub description: Option<String>,

  /// 输入端口定义。与 node 的 outputs 对应。
  #[builder(default, setter(into))]
  pub inputs: Vec<InputPortConfig>,

  /// 输出端口定义。
  #[builder(default, setter(into))]
  pub outputs: Vec<OutputPortConfig>,

  /// 属性定义
  #[builder(default, setter(into))]
  pub properties: Vec<NodeProperty>,

  /// 官方文档URL
  #[builder(default, setter(into, strip_option))]
  pub document_url: Option<String>,

  /// 子标题
  #[builder(default, setter(into, strip_option))]
  pub sub_title: Option<String>,

  /// 是否隐藏
  #[builder(default = default_bool_false())]
  pub hidden: bool,

  /// 节点图标。（支持 FontAwesome 图标或文件图标）或自定义图标URL
  #[builder(default, setter(into, strip_option))]
  pub icon: Option<String>,

  /// 图标颜色
  #[builder(default, setter(into, strip_option))]
  pub icon_color: Option<IconColor>,

  /// 自定义图标URL
  #[builder(default, setter(into, strip_option))]
  pub icon_url: Option<String>,

  /// 徽章图标URL
  #[builder(default, setter(into, strip_option))]
  pub badge_icon_url: Option<String>,
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
