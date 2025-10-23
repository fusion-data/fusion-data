use std::ops::Deref;

use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::Display;
use uuid::Uuid;

use crate::{generate_uuid_newtype, types::JsonValue};

use super::{ExecutionDataMap, NodeName, ValidationError, WorkflowErrorData};

#[derive(Debug, Clone, Default, Serialize, Deserialize, derive_more::From, derive_more::Into)]
#[serde(transparent)]
pub struct ParameterMap(serde_json::Map<String, JsonValue>);

impl From<serde_json::Value> for ParameterMap {
  fn from(value: serde_json::Value) -> Self {
    match value {
      serde_json::Value::Object(map) => Self(map),
      _ => Self::default(),
    }
  }
}

impl Deref for ParameterMap {
  type Target = serde_json::Map<String, JsonValue>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl ParameterMap {
  pub fn new(data: serde_json::Map<String, JsonValue>) -> Self {
    Self(data)
  }

  pub fn get_parameter<T>(&self, field: &str) -> Result<T, ValidationError>
  where
    T: DeserializeOwned,
  {
    let parameter = self.0.get(field).ok_or_else(|| ValidationError::required_field_missing(field))?;
    serde_json::from_value(parameter.clone()).map_err(ValidationError::from)
  }

  pub fn get_optional_parameter<T>(&self, field: &str) -> Option<T>
  where
    T: DeserializeOwned,
  {
    let parameter = self.0.get(field)?;
    serde_json::from_value(parameter.clone()).ok()
  }

  pub fn get<T>(&self) -> Result<T, ValidationError>
  where
    T: DeserializeOwned,
  {
    serde_json::from_value(serde_json::Value::Object(self.0.clone())).map_err(ValidationError::from)
  }

  pub fn get_optional<T>(&self) -> Option<T>
  where
    T: DeserializeOwned,
  {
    serde_json::from_value(serde_json::Value::Object(self.0.clone())).ok()
  }

  pub fn insert(&mut self, field: &str, value: JsonValue) {
    self.0.insert(field.to_string(), value);
  }

  pub fn remove(&mut self, field: &str) {
    self.0.remove(field);
  }

  pub fn into_inner(self) -> serde_json::Map<String, JsonValue> {
    self.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnsureTypeOptions {
  Boolean,
  Number,
  String,
  Array,
  Object,
  Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeParameterOptions {
  /// make sure that returned value would be of specified type, converts it if needed
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ensure_type: Option<EnsureTypeOptions>,

  /// extract value from regex, works only when parameter type is resourceLocator
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub extract_value: Option<bool>,

  /// get raw value of parameter with unresolved expressions
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub raw_expressions: Option<bool>,

  /// skip validation of parameter
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub skip_validation: Option<bool>,
}

impl Default for GetNodeParameterOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl GetNodeParameterOptions {
  pub fn new() -> Self {
    Self { ensure_type: None, extract_value: None, raw_expressions: None, skip_validation: None }
  }

  pub fn extract_value(&self) -> bool {
    self.extract_value.unwrap_or(false)
  }

  pub fn is_raw_expressions(&self) -> bool {
    self.raw_expressions.unwrap_or(false)
  }

  pub fn is_skip_validation(&self) -> bool {
    self.skip_validation.unwrap_or(false)
  }

  // Builder methods
  pub fn with_ensure_type(mut self, ensure_type: EnsureTypeOptions) -> Self {
    self.ensure_type = Some(ensure_type);
    self
  }

  pub fn with_extract_value(mut self, extract_value: bool) -> Self {
    self.extract_value = Some(extract_value);
    self
  }

  pub fn with_raw_expressions(mut self, raw_expressions: bool) -> Self {
    self.raw_expressions = Some(raw_expressions);
    self
  }

  pub fn with_skip_validation(mut self, skip_validation: bool) -> Self {
    self.skip_validation = Some(skip_validation);
    self
  }
}

/// 执行唯一标识符
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  derive_more::Constructor,
  derive_more::Display,
  derive_more::From,
  derive_more::Into,
  derive_more::AsRef,
)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
#[serde(transparent)]
pub struct ExecutionId(Uuid);

/// 工作流唯一标识符
#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  Hash,
  Serialize,
  Deserialize,
  derive_more::Constructor,
  derive_more::Display,
  derive_more::From,
  derive_more::Into,
  derive_more::AsRef,
)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type), sqlx(transparent))]
#[serde(transparent)]
pub struct WorkflowId(Uuid);

generate_uuid_newtype!(Struct: ExecutionId, Struct: WorkflowId);

#[cfg(feature = "with-db")]
fusionsql::generate_uuid_newtype_to_sea_query_value!(Struct: ExecutionId, Struct: WorkflowId);

/// 代码自动完成类型
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum CodeAutocompleteType {
  Function = 1,
  FunctionItem = 2,
}

/// 编辑器类型
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum EditorType {
  CodeNodeEditor = 1,
  JsEditor = 2,
  HtmlEditor = 3,
  SqlEditor = 4,
  CssEditor = 5,
}

/// SQL 方言
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum SqlDialect {
  StandardSQL = 1,
  PostgreSQL = 2,
  MySQL = 3,
  MariaSQL = 4,
  MSSQL = 5,
  SQLite = 6,
  Cassandra = 7,
  PLSQL = 8,
}

/// 按钮动作类型
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ButtonActionKind {
  AskAiCodeGeneration = 1,
}

/// 调用动作类型
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum CalloutActionKind {
  OpenRagStarterTemplate = 1,
}

/// 资源映射器模式
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ResourceMapperMode {
  Add = 1,
  Update = 2,
  Upsert = 3,
  Map = 4,
}

/// 过滤器版本
#[derive(Debug, Clone, Copy, Default, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum FilterVersion {
  #[default]
  V1 = 1,
}

// 类型验证模式
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum TypeValidationMode {
  Strict = 1,
  Loose = 2,
}

/// 过滤器组合器
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum FilterCombinator {
  And = 1,
  Or = 2,
}

/// 字段类型
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum FieldKind {
  Boolean = 1,
  Number = 2,
  String = 3,
  StringAlphanumeric = 4,
  DateTime = 5,
  Time = 6,
  Array = 7,
  Object = 8,
  Options = 9,
  Url = 10,
  Jwt = 11,
  FormFields = 12,
}

/// 显示选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
  pub hide: Option<HashMap<String, Vec<JsonValue>>>,
  pub show: Option<ShowOptions>,
  pub hide_on_cloud: Option<bool>,
}

impl Default for DisplayOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl DisplayOptions {
  pub fn new() -> Self {
    Self { hide: None, show: None, hide_on_cloud: None }
  }

  // Builder methods
  pub fn with_hide<I, K, V>(mut self, hide: I) -> Self
  where
    I: IntoIterator<Item = (K, Vec<V>)>,
    K: Into<String>,
    V: Into<JsonValue>,
  {
    self.hide = Some(hide.into_iter().map(|(k, v)| (k.into(), v.into_iter().map(|v| v.into()).collect())).collect());
    self
  }

  pub fn add_hide(mut self, key: impl Into<String>, values: Vec<impl Into<JsonValue>>) -> Self {
    self
      .hide
      .get_or_insert_with(HashMap::default)
      .insert(key.into(), values.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn with_show(mut self, show: impl Into<ShowOptions>) -> Self {
    self.show = Some(show.into());
    self
  }

  pub fn with_hide_on_cloud(mut self, hide_on_cloud: bool) -> Self {
    self.hide_on_cloud = Some(hide_on_cloud);
    self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowOptions {
  #[serde(rename = "@version")]
  pub version: Option<Vec<i32>>,
  #[serde(rename = "@tool")]
  pub tool: Option<Vec<bool>>,
  #[serde(flatten)]
  pub other: HashMap<String, Vec<JsonValue>>,
}

impl Default for ShowOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl ShowOptions {
  pub fn new() -> Self {
    Self { version: None, tool: None, other: HashMap::default() }
  }

  // Builder methods
  pub fn with_version<I, V>(mut self, version: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<i32>,
  {
    self.version = Some(version.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn add_version(mut self, version: impl Into<i32>) -> Self {
    self.version.get_or_insert_with(Vec::new).push(version.into());
    self
  }

  pub fn with_tool<I, V>(mut self, tool: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<bool>,
  {
    self.tool = Some(tool.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn add_tool(mut self, tool: impl Into<bool>) -> Self {
    self.tool.get_or_insert_with(Vec::new).push(tool.into());
    self
  }

  pub fn with_other<I, K, V>(mut self, other: I) -> Self
  where
    I: IntoIterator<Item = (K, Vec<V>)>,
    K: Into<String>,
    V: Into<JsonValue>,
  {
    self.other = other.into_iter().map(|(k, v)| (k.into(), v.into_iter().map(|v| v.into()).collect())).collect();
    self
  }

  pub fn add_other(mut self, key: impl Into<String>, values: Vec<impl Into<JsonValue>>) -> Self {
    self.other.insert(key.into(), values.into_iter().map(|v| v.into()).collect());
    self
  }
}
/// 按钮动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePropertyAction {
  /// 动作类型
  pub kind: ButtonActionKind,

  /// 处理程序
  pub handler: Option<String>,

  /// 目标
  pub target: Option<String>,
}

impl NodePropertyAction {
  pub fn new(kind: ButtonActionKind) -> Self {
    Self { kind, handler: None, target: None }
  }

  // Builder methods
  pub fn with_kind(mut self, kind: ButtonActionKind) -> Self {
    self.kind = kind;
    self
  }

  pub fn with_handler(mut self, handler: impl Into<String>) -> Self {
    self.handler = Some(handler.into());
    self
  }

  pub fn with_target(mut self, target: impl Into<String>) -> Self {
    self.target = Some(target.into());
    self
  }
}

/// 工作流触发类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TriggerType {
  /// 普通触发，包含节点名称和执行数据
  Normal { node_name: NodeName, execution_data: ExecutionDataMap },
  /// 错误触发，包含错误数据和可选的错误工作流ID
  Error { error_data: Box<WorkflowErrorData>, error_workflow_id: Option<WorkflowId> },
}

/// 统一的工作流触发数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTriggerData {
  /// 触发类型
  pub trigger_type: TriggerType,
}

impl WorkflowTriggerData {
  /// 创建正常触发的工作流数据
  pub fn normal(node_name: NodeName, execution_data: ExecutionDataMap) -> Self {
    Self { trigger_type: TriggerType::Normal { node_name, execution_data } }
  }

  /// 创建错误触发的工作流数据
  pub fn error(error_data: WorkflowErrorData, error_workflow_id: Option<WorkflowId>) -> Self {
    Self { trigger_type: TriggerType::Error { error_data: Box::new(error_data), error_workflow_id } }
  }

  /// 从旧的元组格式转换（向后兼容）
  pub fn from_tuple(trigger_data: (NodeName, ExecutionDataMap)) -> Self {
    Self::normal(trigger_data.0, trigger_data.1)
  }

  /// 转换为旧的元组格式（向后兼容）
  pub fn to_tuple(&self) -> Option<(NodeName, ExecutionDataMap)> {
    match &self.trigger_type {
      TriggerType::Normal { node_name, execution_data } => Some((node_name.clone(), execution_data.clone())),
      TriggerType::Error { .. } => None, // 错误类型无法转换为元组格式
    }
  }

  /// 获取错误数据（如果是错误触发）
  pub fn get_error_data(&self) -> Option<(&WorkflowErrorData, Option<&WorkflowId>)> {
    match &self.trigger_type {
      TriggerType::Error { error_data, error_workflow_id } => Some((error_data, error_workflow_id.as_ref())),
      TriggerType::Normal { .. } => None,
    }
  }

  /// 检查是否为正常触发
  pub fn is_normal(&self) -> bool {
    matches!(self.trigger_type, TriggerType::Normal { .. })
  }

  /// 检查是否为错误触发
  pub fn is_error(&self) -> bool {
    matches!(self.trigger_type, TriggerType::Error { .. })
  }
}

/// 按钮配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonConfig {
  pub action: NodePropertyAction,

  /// 标签
  pub label: Option<String>,

  /// 是否包含输入字段
  pub has_input_field: Option<bool>,

  /// 输入字段最大长度。has_input_field 为 true 时有效。
  pub input_field_max_length: Option<i32>,
}

impl ButtonConfig {
  pub fn new(action: NodePropertyAction) -> Self {
    Self { action, label: None, has_input_field: None, input_field_max_length: None }
  }

  // Builder methods
  pub fn with_action(mut self, action: impl Into<NodePropertyAction>) -> Self {
    self.action = action.into();
    self
  }

  pub fn with_label(mut self, label: impl Into<String>) -> Self {
    self.label = Some(label.into());
    self
  }

  pub fn with_has_input_field(mut self, has_input_field: bool) -> Self {
    self.has_input_field = Some(has_input_field);
    self
  }

  pub fn with_input_field_max_length(mut self, input_field_max_length: i32) -> Self {
    self.input_field_max_length = Some(input_field_max_length);
    self
  }
}

// 加载选项配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadOptions {
  // 简化结构，实际包含路由配置等
  pub routing: Option<HashMap<String, JsonValue>>,
}

impl LoadOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_routing<I, K, V>(mut self, routing: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<JsonValue>,
  {
    self.routing = Some(routing.into_iter().map(|(k, v)| (k.into(), v.into())).collect());
    self
  }

  pub fn add_routing(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
    self.routing.get_or_insert_with(HashMap::default).insert(key.into(), value.into());
    self
  }
}

// 调用动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutAction {
  pub action_kind: CalloutActionKind,
  pub label: String,
}

impl CalloutAction {
  pub fn new(action_kind: CalloutActionKind, label: impl Into<String>) -> Self {
    Self { action_kind, label: label.into() }
  }

  // Builder methods
  pub fn with_action_kind(mut self, action_kind: CalloutActionKind) -> Self {
    self.action_kind = action_kind;
    self
  }

  pub fn with_label(mut self, label: impl Into<String>) -> Self {
    self.label = label.into();
    self
  }
}

// 资源映射器字段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMapperFieldWords {
  pub singular: String,
  pub plural: String,
}

impl ResourceMapperFieldWords {
  pub fn new(singular: impl Into<String>, plural: impl Into<String>) -> Self {
    Self { singular: singular.into(), plural: plural.into() }
  }

  // Builder methods
  pub fn with_singular(mut self, singular: impl Into<String>) -> Self {
    self.singular = singular.into();
    self
  }

  pub fn with_plural(mut self, plural: impl Into<String>) -> Self {
    self.plural = plural.into();
    self
  }
}

// 匹配字段标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingFieldsLabels {
  pub title: Option<String>,
  pub description: Option<String>,
  pub hint: Option<String>,
}

impl Default for MatchingFieldsLabels {
  fn default() -> Self {
    Self::new()
  }
}

impl MatchingFieldsLabels {
  pub fn new() -> Self {
    Self { title: None, description: None, hint: None }
  }

  // Builder methods
  pub fn with_title(mut self, title: impl Into<String>) -> Self {
    self.title = Some(title.into());
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
}

// 资源映射器类型选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMapperTypeOptions {
  pub mode: ResourceMapperMode,
  pub values_label: Option<String>,
  pub field_words: Option<ResourceMapperFieldWords>,
  pub add_all_fields: Option<bool>,
  pub no_fields_error: Option<String>,
  pub multi_key_match: Option<bool>,
  pub support_auto_map: Option<bool>,
  pub matching_fields_labels: Option<MatchingFieldsLabels>,
  pub show_type_conversion_options: Option<bool>,
  pub resource_mapper_method: Option<String>,
  pub local_resource_mapper_method: Option<String>,
}

impl ResourceMapperTypeOptions {
  pub fn new(mode: ResourceMapperMode) -> Self {
    Self {
      mode,
      values_label: None,
      field_words: None,
      add_all_fields: None,
      no_fields_error: None,
      multi_key_match: None,
      support_auto_map: None,
      matching_fields_labels: None,
      show_type_conversion_options: None,
      resource_mapper_method: None,
      local_resource_mapper_method: None,
    }
  }

  // Builder methods
  pub fn with_mode(mut self, mode: ResourceMapperMode) -> Self {
    self.mode = mode;
    self
  }

  pub fn with_values_label(mut self, values_label: impl Into<String>) -> Self {
    self.values_label = Some(values_label.into());
    self
  }

  pub fn with_field_words(mut self, field_words: impl Into<ResourceMapperFieldWords>) -> Self {
    self.field_words = Some(field_words.into());
    self
  }

  pub fn with_add_all_fields(mut self, add_all_fields: bool) -> Self {
    self.add_all_fields = Some(add_all_fields);
    self
  }

  pub fn with_no_fields_error(mut self, no_fields_error: impl Into<String>) -> Self {
    self.no_fields_error = Some(no_fields_error.into());
    self
  }

  pub fn with_multi_key_match(mut self, multi_key_match: bool) -> Self {
    self.multi_key_match = Some(multi_key_match);
    self
  }

  pub fn with_support_auto_map(mut self, support_auto_map: bool) -> Self {
    self.support_auto_map = Some(support_auto_map);
    self
  }

  pub fn with_matching_fields_labels(mut self, matching_fields_labels: impl Into<MatchingFieldsLabels>) -> Self {
    self.matching_fields_labels = Some(matching_fields_labels.into());
    self
  }

  pub fn with_show_type_conversion_options(mut self, show_type_conversion_options: bool) -> Self {
    self.show_type_conversion_options = Some(show_type_conversion_options);
    self
  }

  pub fn with_resource_mapper_method(mut self, resource_mapper_method: impl Into<String>) -> Self {
    self.resource_mapper_method = Some(resource_mapper_method.into());
    self
  }

  pub fn with_local_resource_mapper_method(mut self, local_resource_mapper_method: impl Into<String>) -> Self {
    self.local_resource_mapper_method = Some(local_resource_mapper_method.into());
    self
  }
}

// 过滤器类型选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTypeOptions {
  pub version: FilterVersion,

  /// 是否区分大小写。可以为 true/false 或表达式，表达式结果需要为 boolean 类型。
  pub case_sensitive: Option<JsonValue>,

  pub left_value: Option<String>,

  pub allowed_combinators: Option<Vec<FilterCombinator>>,

  pub max_conditions: Option<i32>,

  pub type_validation: Option<TypeValidationMode>,
}

impl Default for FilterTypeOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl FilterTypeOptions {
  pub fn new() -> Self {
    Self {
      version: FilterVersion::default(),
      case_sensitive: None,
      left_value: None,
      allowed_combinators: None,
      max_conditions: None,
      type_validation: None,
    }
  }

  // Builder methods
  pub fn with_version(mut self, version: FilterVersion) -> Self {
    self.version = version;
    self
  }

  pub fn with_case_sensitive(mut self, case_sensitive: JsonValue) -> Self {
    self.case_sensitive = Some(case_sensitive);
    self
  }

  pub fn with_left_value(mut self, left_value: impl Into<String>) -> Self {
    self.left_value = Some(left_value.into());
    self
  }

  pub fn with_allowed_combinators<I, V>(mut self, allowed_combinators: I) -> Self
  where
    I: IntoIterator<Item = V>,
    V: Into<FilterCombinator>,
  {
    self.allowed_combinators = Some(allowed_combinators.into_iter().map(|v| v.into()).collect());
    self
  }

  pub fn add_allowed_combinator(mut self, allowed_combinator: FilterCombinator) -> Self {
    self.allowed_combinators.get_or_insert_with(Vec::new).push(allowed_combinator);
    self
  }

  pub fn with_max_conditions(mut self, max_conditions: i32) -> Self {
    self.max_conditions = Some(max_conditions);
    self
  }

  pub fn with_type_validation(mut self, type_validation: TypeValidationMode) -> Self {
    self.type_validation = Some(type_validation);
    self
  }
}

// 赋值类型选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentKindOptions {
  pub hide_kind: Option<bool>,
  pub default_kind: Option<FieldKind>,
  pub disable_kind: Option<bool>,
}

impl Default for AssignmentKindOptions {
  fn default() -> Self {
    Self::new()
  }
}

impl AssignmentKindOptions {
  pub fn new() -> Self {
    Self { hide_kind: None, default_kind: None, disable_kind: None }
  }

  // Builder methods
  pub fn with_hide_kind(mut self, hide_kind: bool) -> Self {
    self.hide_kind = Some(hide_kind);
    self
  }

  pub fn with_default_kind(mut self, default_kind: FieldKind) -> Self {
    self.default_kind = Some(default_kind);
    self
  }

  pub fn with_disable_kind(mut self, disable_kind: bool) -> Self {
    self.disable_kind = Some(disable_kind);
    self
  }
}

/// 字段类型枚举
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum FieldType {
  Boolean = 1,
  Number = 2,
  String = 3,
  StringAlphanumeric = 4,
  DateTime = 5,
  Time = 6,
  Array = 7,
  Object = 8,
  Options = 9,
  Url = 10,
  Jwt = 11,
  FormFields = 12,
}

/// 数据路径要求枚举
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum DataPathRequirement {
  Single = 1,
  Multiple = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Display)]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
#[repr(i32)]
pub enum CredentialKind {
  GenericAuth = 1,
  Authenticate = 2,
  Oauth2 = 3,
}

#[derive(Debug)]
pub enum NodeParameterValueType {
  String(String),
  Number(f64),
  Boolean(bool),
  Object(HashMap<String, Box<NodeParameterValueType>>),
}
