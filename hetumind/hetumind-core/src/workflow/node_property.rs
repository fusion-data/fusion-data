use serde::{Deserialize, Serialize};

use crate::{
  types::JsonValue,
  workflow::{
    AssignmentKindOptions, ButtonConfig, CalloutAction, CodeAutocompleteType, CredentialKind, DataPathRequirement,
    DisplayOptions, EditorType, FieldType, FilterTypeOptions, LoadOptions, ResourceMapperTypeOptions, SqlDialect,
  },
};

/// Node property type (metadata)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodePropertyKind {
  #[default]
  String,
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
  WorkflowSelector,
}

// Node property type options - main structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePropertyKindOptions {
  /// 按钮配置 (支持: [NodePropertyKind::Button])
  pub button_config: Option<ButtonConfig>,

  /// 容器类名 (支持: [NodePropertyKind::Notice])
  pub container_class: Option<String>,

  /// 始终打开编辑窗口 (支持: [NodePropertyKind::Json])
  pub always_open_edit_window: Option<bool>,

  /// 代码自动完成 (支持: [NodePropertyKind::String])
  pub code_autocomplete: Option<CodeAutocompleteType>,

  /// 编辑器类型 (支持: [NodePropertyKind::String])
  pub editor: Option<EditorType>,

  /// 编辑器只读 (支持: [NodePropertyKind::String])
  pub editor_is_read_only: Option<bool>,

  /// SQL 方言 (支持: [EditorType::SqlEditor])
  pub sql_dialect: Option<SqlDialect>,

  /// 加载选项依赖 (支持: [NodePropertyKind::Options])
  pub load_options_depends_on: Option<Vec<String>>,

  /// 加载选项方法 (支持: [NodePropertyKind::Options])
  pub load_options_method: Option<String>,

  /// 加载选项 (支持: [NodePropertyKind::Options])
  pub load_options: Option<LoadOptions>,

  /// 最大值 (支持: [NodePropertyKind::Number])
  pub max_value: Option<f64>,

  /// 最小值 (支持: [NodePropertyKind::Number])
  pub min_value: Option<f64>,

  /// 多个值 (支持: all [NodePropertyKind])
  pub multiple_values: Option<bool>,

  /// 多值按钮文本 (当 [Self::multiple_values]=true 时支持)
  pub multiple_value_button_text: Option<String>,

  /// 数字精度 (支持: [NodePropertyKind::Number])
  pub number_precision: Option<i32>,

  /// 密码字段 (支持: [NodePropertyKind::String])
  pub password: Option<bool>,

  /// 行数 (支持: [NodePropertyKind::String])
  pub rows: Option<i32>,

  /// 显示透明度 (支持: [NodePropertyKind::Color])
  pub show_alpha: Option<bool>,

  /// 可排序 (当 [Self::multiple_values]=true 时支持)
  pub sortable: Option<bool>,

  /// 可过期 (支持: [NodePropertyKind::Hidden]，仅在凭据中)
  pub expirable: Option<bool>,

  /// 资源映射器配置
  pub resource_mapper: Option<ResourceMapperTypeOptions>,

  /// 过滤器配置
  pub filter: Option<FilterTypeOptions>,

  /// 赋值配置
  pub assignment: Option<AssignmentKindOptions>,

  /// 最少必需字段数 (支持: [NodePropertyKind::FixedCollection])
  pub min_required_fields: Option<i32>,

  /// 最多允许字段数 (支持: [NodePropertyKind::FixedCollection])
  pub max_allowed_fields: Option<i32>,

  /// 调用动作 (支持: [NodePropertyKind::Callout])
  pub callout_action: Option<CalloutAction>,

  /// 其他扩展字段
  #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeProperty {
  /// 在节点定义中唯一标识一个属性。实际工作流使用时，由工作流节点的 parameters 中的 key 表示。
  pub name: String,

  /// 显示名称
  pub display_name: String,

  /// 属性类型。UI 渲染方式依赖于此
  pub kind: NodePropertyKind,

  /// 针对特定类型的附加配置选项
  pub kind_options: Option<NodePropertyKindOptions>,

  /// 是否必填
  pub required: bool,

  /// 参数的默认值
  pub value: Option<JsonValue>,

  /// 参数的描述信息
  pub description: Option<String>,

  /// 参数的提示信息
  pub hint: Option<String>,

  /// 禁用选项
  pub disable_options: Option<DisplayOptions>,

  /// 显示选项
  pub display_options: Option<DisplayOptions>,

  /// 选项
  pub options: Option<Vec<Box<NodeProperty>>>,

  /// 输入框占位符文本
  pub placeholder: Option<String>,

  /// 是否为节点级别设置
  pub is_node_setting: Option<bool>,

  /// 是否禁止使用数据表达式
  pub no_data_expression: Option<bool>,

  /// API 路由配置
  pub routing: Option<NodePropertyRouting>,

  /// 支持的凭据类型列表
  pub credential_kinds: Option<Vec<CredentialKind>>,

  /// 值提取配置
  pub extract_value: Option<NodePropertyValueExtractor>,

  /// 参数的不同模式配置
  pub modes: Option<Vec<NodePropertyMode>>,

  /// 需要的数据路径类型
  pub requires_data_path: Option<DataPathRequirement>,

  /// 是否不从父级继承此参数
  pub do_not_inherit: Option<bool>,

  /// 用于验证和类型转换的预期类型
  pub validate_type: Option<FieldType>,

  /// 执行期间是否跳过验证
  pub ignore_validation_during_execution: Option<bool>,

  /// 是否是密码类型？
  pub password: Option<bool>,

  /// 扩展属性
  #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
  pub additional_properties: serde_json::Map<String, JsonValue>,
}

impl NodeProperty {
  pub fn new(kind: NodePropertyKind) -> Self {
    Self::new_option_value(JsonValue::Null, kind)
  }

  /// Create a new optional node property with default name and display name
  pub fn new_option_value(value: JsonValue, kind: NodePropertyKind) -> Self {
    let name = serde_json::to_string(&value).unwrap();
    Self::new_option(&name, &name, value, kind)
  }

  /// Create a new optional node property
  pub fn new_option(
    display_name: impl Into<String>,
    name: impl Into<String>,
    value: JsonValue,
    kind: NodePropertyKind,
  ) -> Self {
    Self {
      display_name: display_name.into(),
      name: name.into(),
      value: Some(value),
      kind,
      options: None,
      ignore_validation_during_execution: None,
      password: None,
      additional_properties: serde_json::Map::new(),
      kind_options: Default::default(),
      required: Default::default(),
      description: Default::default(),
      hint: Default::default(),
      disable_options: Default::default(),
      display_options: Default::default(),
      placeholder: Default::default(),
      is_node_setting: Default::default(),
      no_data_expression: Default::default(),
      routing: Default::default(),
      credential_kinds: Default::default(),
      extract_value: Default::default(),
      modes: Default::default(),
      requires_data_path: Default::default(),
      do_not_inherit: Default::default(),
      validate_type: Default::default(),
    }
  }

  pub fn with_options<I, V>(mut self, options: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<Box<NodeProperty>>,
  {
    self.options = Some(options.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn add_option(mut self, option: impl Into<Box<NodeProperty>>) -> Self {
    self.options.get_or_insert_with(Vec::new).push(option.into());
    self
  }

  // Simple and Option<T> field modifiers
  pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
    self.display_name = display_name.into();
    self
  }

  pub fn with_name(mut self, name: impl Into<String>) -> Self {
    self.name = name.into();
    self
  }

  pub fn with_kind(mut self, kind: NodePropertyKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_kind_options(mut self, kind_options: impl Into<NodePropertyKindOptions>) -> Self {
    self.kind_options = Some(kind_options.into());
    self
  }

  pub fn with_required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }

  pub fn with_value(mut self, value: impl Into<JsonValue>) -> Self {
    self.value = Some(value.into());
    self
  }

  pub fn with_description(mut self, description: impl Into<String>) -> Self {
    self.description = Some(description.into());
    self
  }

  pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
    self.hint = Some(hint.into());
    self
  }

  pub fn with_disable_options(mut self, disable_options: impl Into<DisplayOptions>) -> Self {
    self.disable_options = Some(disable_options.into());
    self
  }

  pub fn with_display_options(mut self, display_options: impl Into<DisplayOptions>) -> Self {
    self.display_options = Some(display_options.into());
    self
  }

  pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
    self.placeholder = Some(placeholder.into());
    self
  }

  pub fn with_is_node_setting(mut self, is_node_setting: bool) -> Self {
    self.is_node_setting = Some(is_node_setting);
    self
  }

  pub fn with_no_data_expression(mut self, no_data_expression: bool) -> Self {
    self.no_data_expression = Some(no_data_expression);
    self
  }

  pub fn with_routing(mut self, routing: impl Into<NodePropertyRouting>) -> Self {
    self.routing = Some(routing.into());
    self
  }

  pub fn with_credential_kinds<I, V>(mut self, credential_kinds: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<CredentialKind>,
  {
    self.credential_kinds = Some(credential_kinds.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn add_credential_kind(mut self, credential_kind: impl Into<CredentialKind>) -> Self {
    self.credential_kinds.get_or_insert_with(Vec::new).push(credential_kind.into());
    self
  }

  pub fn with_extract_value(mut self, extract_value: impl Into<NodePropertyValueExtractor>) -> Self {
    self.extract_value = Some(extract_value.into());
    self
  }

  pub fn with_modes<I, V>(mut self, modes: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<NodePropertyMode>,
  {
    self.modes = Some(modes.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn add_mode(mut self, mode: impl Into<NodePropertyMode>) -> Self {
    self.modes.get_or_insert_with(Vec::new).push(mode.into());
    self
  }

  pub fn with_requires_data_path(mut self, requires_data_path: impl Into<DataPathRequirement>) -> Self {
    self.requires_data_path = Some(requires_data_path.into());
    self
  }

  pub fn with_do_not_inherit(mut self, do_not_inherit: bool) -> Self {
    self.do_not_inherit = Some(do_not_inherit);
    self
  }

  pub fn with_validate_type(mut self, validate_type: impl Into<FieldType>) -> Self {
    self.validate_type = Some(validate_type.into());
    self
  }

  pub fn with_ignore_validation_during_execution(mut self, ignore_validation_during_execution: bool) -> Self {
    self.ignore_validation_during_execution = Some(ignore_validation_during_execution);
    self
  }

  pub fn with_password(mut self, password: bool) -> Self {
    self.password = Some(password);
    self
  }

  pub fn with_additional_property<K, V>(mut self, key: K, value: V) -> Self
  where
    K: Into<String>,
    V: Into<JsonValue>,
  {
    self.additional_properties.insert(key.into(), value.into());
    self
  }

  pub fn with_additional_properties<I, K, V>(mut self, properties: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<JsonValue>,
  {
    for (key, value) in properties {
      self.additional_properties.insert(key.into(), value.into());
    }
    self
  }
}
