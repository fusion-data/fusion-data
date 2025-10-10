use std::ops::Deref;

use fusion_common::ahash::HashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{AsRefStr, Display};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{generate_uuid_newtype, types::JsonValue};

use super::ValidationError;

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

impl GetNodeParameterOptions {
  pub fn extract_value(&self) -> bool {
    self.extract_value.unwrap_or(false)
  }

  pub fn is_raw_expressions(&self) -> bool {
    self.raw_expressions.unwrap_or(false)
  }

  pub fn is_skip_validation(&self) -> bool {
    self.skip_validation.unwrap_or(false)
  }
}

/// 执行唯一标识符
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowOptions {
  #[serde(rename = "@version")]
  pub version: Option<Vec<i32>>,
  #[serde(rename = "@tool")]
  pub tool: Option<Vec<bool>>,
  #[serde(flatten)]
  pub other: HashMap<String, Vec<JsonValue>>,
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

/// 按钮配置
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct ButtonConfig {
  pub action: NodePropertyAction,

  /// 标签
  #[builder(default, setter(into, strip_option))]
  pub label: Option<String>,

  /// 是否包含输入字段
  #[builder(default, setter(strip_option))]
  pub has_input_field: Option<bool>,

  /// 输入字段最大长度。has_input_field 为 true 时有效。
  #[builder(default, setter(strip_option))]
  pub input_field_max_length: Option<i32>,
}

// 加载选项配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOptions {
  // 简化结构，实际包含路由配置等
  pub routing: Option<HashMap<String, JsonValue>>,
}

// 调用动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutAction {
  pub action_kind: CalloutActionKind,
  pub label: String,
}

// 资源映射器字段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMapperFieldWords {
  pub singular: String,
  pub plural: String,
}

// 匹配字段标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingFieldsLabels {
  pub title: Option<String>,
  pub description: Option<String>,
  pub hint: Option<String>,
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

// 过滤器类型选项
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct FilterTypeOptions {
  #[builder(default)]
  pub version: FilterVersion,

  /// 是否区分大小写。可以为 true/false 或表达式，表达式结果需要为 boolean 类型。
  #[builder(default, setter(strip_option))]
  pub case_sensitive: Option<JsonValue>,

  #[builder(default, setter(strip_option))]
  pub left_value: Option<String>,

  #[builder(default, setter(into, strip_option))]
  pub allowed_combinators: Option<Vec<FilterCombinator>>,

  #[builder(default, setter(strip_option))]
  pub max_conditions: Option<i32>,

  #[builder(default, setter(strip_option))]
  pub type_validation: Option<TypeValidationMode>,
}

// 赋值类型选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentKindOptions {
  pub hide_kind: Option<bool>,
  pub default_kind: Option<FieldKind>,
  pub disable_kind: Option<bool>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(feature = "with-db", derive(sqlx::Type))]
pub enum CredentialKind {
  Oauth2,
  Authenticate,
  GenericAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryFileKind {
  Text,
  Json,
  Image,
  Video,
  Audio,
  Pdf,
  Html,
  Excel,
  Word,
  Ppt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFileType {
  Csv,
  Markdown,
  Xml,
  Yaml,
  Toml,
  Properties,
  Ini,
}

#[derive(Debug)]
pub enum NodeParameterValueType {
  String(String),
  Number(f64),
  Boolean(bool),
  Object(HashMap<String, Box<NodeParameterValueType>>),
}

#[cfg(feature = "with-db")]
fusionsql::generate_enum_string_to_sea_query_value!(
  Enum: CredentialKind,
);
