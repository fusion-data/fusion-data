//! Edit Fields 数据编辑节点实现
//!
//! 参考 n8n 的 Edit Fields 节点设计（v3.4），用于编辑、修改或删除数据字段。
//! 支持两种操作模式：Manual Mapping（手动映射）和 JSON（自定义JSON），
//! 提供灵活的字段操作能力和丰富的输出控制选项。
//!
//! # 主要功能特性
//! - **双模式操作**: Manual Mapping（手动映射）和 JSON（自定义JSON）两种模式
//! - **灵活字段操作**: 支持添加、修改、删除、复制、增加、追加等多种操作
//! - **多种数据类型**: String、Number、Boolean、Array、Object 及其转换
//! - **输出控制**: 四种包含模式（全部、无、选定、排除除外）
//! - **点表示法支持**: 支持嵌套字段访问和设置
//! - **二进制数据处理**: 可选择包含或剥离二进制数据
//! - **类型转换控制**: 可忽略类型转换错误以提供更灵活的处理
//! - **项目复制功能**: 支持测试和调试时的项目复制
//!
//! # 操作类型
//! - `Set`: 设置字段值
//! - `Remove`: 删除字段
//! - `Copy`: 从其他字段复制值
//! - `Increment`: 数值增加
//! - `Append`: 数组追加元素
//! - `Prepend`: 数组前置元素
//! - `Multiply`: 数值乘法
//! - `Replace`: 字符串替换
//! - `Split`: 字符串分割
//! - `Join`: 数组连接
//!
//! # 数据来源
//! - `Static`: 静态值
//! - `Expression`: 表达式（如 $.field.subfield）
//! - `CurrentTimestamp`: 当前时间戳
//! - `Random`: 随机值
//! - `Uuid`: UUID 值
//! - `JsonPath`: JSONPath 表达式

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod edit_fields_v1;
mod utils;

use edit_fields_v1::EditFieldsV1;

use crate::constants::EDIT_FIELDS_NODE_KIND;

/// 操作模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationMode {
  /// 手动映射模式
  Manual,
  /// JSON 模式
  Json,
}

/// 操作类型 - 扩展版本
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
  /// 设置字段值
  Set,
  /// 删除字段
  Remove,
  /// 从其他字段复制值
  Copy,
  /// 数值增加
  Increment,
  /// 数组追加元素
  Append,
  /// 数组前置元素
  Prepend,
  /// 数值乘法
  Multiply,
  /// 字符串替换
  Replace,
  /// 字符串分割为数组
  Split,
  /// 数组连接为字符串
  Join,
}

/// 数据来源类型 - 扩展版本
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueSourceKind {
  /// 静态值
  Static,
  /// 表达式（JSON Path）
  Expression,
  /// 当前时间戳
  CurrentTimestamp,
  /// 随机值
  Random,
  /// UUID 值
  Uuid,
  /// JSONPath 表达式
  JsonPath,
  /// 环境变量
  EnvironmentVariable,
}

/// 字段类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
  /// 字符串类型
  String,
  /// 数值类型
  Number,
  /// 布尔类型
  Boolean,
  /// 数组类型
  Array,
  /// 对象类型
  Object,
  /// 自动检测类型
  Auto,
}

/// 输出包含模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncludeMode {
  /// 包含所有输入字段
  All,
  /// 不包含任何输入字段
  None,
  /// 仅包含选定字段
  Selected,
  /// 包含除指定字段外的所有字段
  Except,
}

/// 二进制数据处理模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryDataMode {
  /// 包含二进制数据
  Include,
  /// 剥离二进制数据
  Strip,
  /// 自动处理
  Auto,
}

/// 字段操作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOperation {
  /// 目标字段路径（支持点表示法）
  pub field_path: String,
  /// 操作类型
  pub operation: OperationKind,
  /// 值来源类型
  pub value_source: ValueSourceKind,
  /// 设置的值（当 value_source 为 Static 或 Expression 时使用）
  pub value: Option<Value>,
  /// 字段类型
  pub field_type: Option<FieldType>,
  /// 操作参数（用于特定操作，如 Replace、Split 等）
  pub operation_params: Option<Value>,
  /// 是否保持原始类型
  pub keep_original_type: Option<bool>,
  /// 是否忽略转换错误
  pub ignore_conversion_error: Option<bool>,
}

impl FieldOperation {
  /// 验证操作配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.field_path.trim().is_empty() {
      return Err("Field path cannot be empty".to_string());
    }

    // 验证字段路径格式
    if self.field_path.contains("..") {
      return Err("Field path cannot contain consecutive dots".to_string());
    }

    if self.field_path.starts_with('.') || self.field_path.ends_with('.') {
      return Err("Field path cannot start or end with a dot".to_string());
    }

    match self.operation {
      OperationKind::Set
      | OperationKind::Copy
      | OperationKind::Increment
      | OperationKind::Append
      | OperationKind::Prepend
      | OperationKind::Multiply
      | OperationKind::Replace
      | OperationKind::Split
      | OperationKind::Join => {
        if self.value_source == ValueSourceKind::Static && self.value.is_none() {
          return Err("Static value source requires a value".to_string());
        }
        if self.value_source == ValueSourceKind::Expression
          && self.value.as_ref().and_then(|v| v.as_str()).is_none_or(|s| s.trim().is_empty())
        {
          return Err("Expression value source requires a valid expression".to_string());
        }
      }
      OperationKind::Remove => {
        // Remove 操作不需要值
      }
    }

    // 验证特定操作的参数
    if let Some(params) = &self.operation_params {
      match self.operation {
        OperationKind::Replace => {
          if !params.is_object() || params.get("search_string").is_none() {
            return Err("Replace operation requires 'search_string' parameter".to_string());
          }
        }
        OperationKind::Split => {
          if !params.is_object() || params.get("separator").is_none() {
            return Err("Split operation requires 'separator' parameter".to_string());
          }
        }
        OperationKind::Join => {
          if !params.is_object() || params.get("separator").is_none() {
            return Err("Join operation requires 'separator' parameter".to_string());
          }
        }
        _ => {}
      }
    }

    Ok(())
  }
}

/// JSON 模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonModeConfig {
  /// JSON 输出模板（支持表达式）
  pub json_output: String,
  /// 是否使用表达式
  pub use_expressions: bool,
  /// 是否验证 JSON 格式
  pub validate_json: bool,
  /// 错误处理策略
  pub error_handling: JsonErrorHandling,
}

/// JSON 错误处理策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonErrorHandling {
  /// 停止执行
  StopExecution,
  /// 使用原始数据
  UseOriginalData,
  /// 返回错误信息
  ReturnError,
  /// 跳过当前项
  SkipItem,
}

/// 手动映射模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualModeConfig {
  /// 字段操作列表
  pub fields: Vec<FieldOperation>,
  /// 输出包含模式
  pub include_mode: IncludeMode,
  /// 选定字段列表（用于 Selected 和 Except 模式）
  pub selected_fields: Option<Vec<String>>,
}

/// Edit Fields 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFieldsConfig {
  /// 操作模式
  pub mode: OperationMode,
  /// 手动映射配置
  pub manual_config: Option<ManualModeConfig>,
  /// JSON 模式配置
  pub json_config: Option<JsonModeConfig>,
  /// 选项配置
  pub options: EditFieldsOptions,
}

/// 节点选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFieldsOptions {
  /// 是否支持点表示法
  pub dot_notation: Option<bool>,
  /// 是否忽略类型转换错误
  pub ignore_conversion_errors: Option<bool>,
  /// 二进制数据处理模式
  pub binary_data_mode: Option<BinaryDataMode>,
  /// 是否保留原始类型
  pub keep_original_type: Option<bool>,
  /// 项目复制功能
  pub duplicate_item: Option<bool>,
  /// 复制数量
  pub duplicate_count: Option<usize>,
  /// 调试模式
  pub debug_mode: Option<bool>,
}

impl Default for EditFieldsOptions {
  fn default() -> Self {
    Self {
      dot_notation: Some(true),
      ignore_conversion_errors: Some(false),
      binary_data_mode: Some(BinaryDataMode::Auto),
      keep_original_type: Some(false),
      duplicate_item: Some(false),
      duplicate_count: Some(0),
      debug_mode: Some(false),
    }
  }
}

impl EditFieldsConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    match self.mode {
      OperationMode::Manual => {
        if self.manual_config.is_none() {
          return Err("Manual mode requires manual_config".to_string());
        }
        if let Some(manual_config) = &self.manual_config {
          for (index, field) in manual_config.fields.iter().enumerate() {
            field.validate().map_err(|e| format!("Invalid field operation at index {}: {}", index, e))?;
          }

          // 验证包含模式
          if matches!(manual_config.include_mode, IncludeMode::Selected | IncludeMode::Except)
            && manual_config.selected_fields.is_none()
          {
            return Err("Selected or Except mode requires selected_fields".to_string());
          }
        }
      }
      OperationMode::Json => {
        if self.json_config.is_none() {
          return Err("JSON mode requires json_config".to_string());
        }
        if let Some(json_config) = &self.json_config
          && json_config.json_output.trim().is_empty()
        {
          return Err("JSON output template cannot be empty".to_string());
        }
      }
    }

    // 验证复制选项
    if let Some(true) = self.options.duplicate_item
      && let Some(count) = self.options.duplicate_count
      && count > 100
    {
      return Err("Duplicate count cannot exceed 100".to_string());
    }

    Ok(())
  }
}

/// Edit Fields 节点实现
pub struct EditFieldsNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl EditFieldsNode {
  /// 创建新的 EditFields 节点
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(EditFieldsV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(EDIT_FIELDS_NODE_KIND, Version::new(1, 0, 0), "Edit Fields")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description("Edit, modify, or delete data fields. Supports both manual mapping and JSON modes with advanced output control options.")
      .with_icon("edit")
  }
}

impl Node for EditFieldsNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};
  use serde_json::json;

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = EditFieldsNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), EDIT_FIELDS_NODE_KIND);
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Edit Fields");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = EditFieldsNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_field_operation_validation() {
    // 有效的设置操作
    let valid_set_op = FieldOperation {
      field_path: "user.name".to_string(),
      operation: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: Some(Value::String("John".to_string())),
      field_type: Some(FieldType::String),
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };
    assert!(valid_set_op.validate().is_ok());

    // 无效的空字段路径
    let invalid_path_op = FieldOperation {
      field_path: "".to_string(),
      operation: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: Some(Value::String("test".to_string())),
      field_type: None,
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };
    assert!(invalid_path_op.validate().is_err());

    // 无效的静态值来源
    let invalid_static_op = FieldOperation {
      field_path: "test".to_string(),
      operation: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: None,
      field_type: None,
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };
    assert!(invalid_static_op.validate().is_err());

    // 有效的删除操作（不需要值）
    let valid_remove_op = FieldOperation {
      field_path: "user.temp".to_string(),
      operation: OperationKind::Remove,
      value_source: ValueSourceKind::Static,
      value: None,
      field_type: None,
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };
    assert!(valid_remove_op.validate().is_ok());

    // 有效的 Replace 操作（带参数）
    let valid_replace_op = FieldOperation {
      field_path: "description".to_string(),
      operation: OperationKind::Replace,
      value_source: ValueSourceKind::Static,
      value: Some(Value::String("replacement".to_string())),
      field_type: Some(FieldType::String),
      operation_params: Some(json!({
        "search_string": "old",
        "replace_string": "new",
        "case_sensitive": false
      })),
      keep_original_type: None,
      ignore_conversion_error: None,
    };
    assert!(valid_replace_op.validate().is_ok());
  }

  #[test]
  fn test_config_validation() {
    // 有效的手动模式配置
    let valid_manual_config = EditFieldsConfig {
      mode: OperationMode::Manual,
      manual_config: Some(ManualModeConfig {
        fields: vec![FieldOperation {
          field_path: "name".to_string(),
          operation: OperationKind::Set,
          value_source: ValueSourceKind::Static,
          value: Some(Value::String("John".to_string())),
          field_type: Some(FieldType::String),
          operation_params: None,
          keep_original_type: None,
          ignore_conversion_error: None,
        }],
        include_mode: IncludeMode::All,
        selected_fields: None,
      }),
      json_config: None,
      options: EditFieldsOptions::default(),
    };
    assert!(valid_manual_config.validate().is_ok());

    // 无效的手动模式配置（缺少 manual_config）
    let invalid_manual_config = EditFieldsConfig {
      mode: OperationMode::Manual,
      manual_config: None,
      json_config: None,
      options: EditFieldsOptions::default(),
    };
    assert!(invalid_manual_config.validate().is_err());

    // 有效的 JSON 模式配置
    let valid_json_config = EditFieldsConfig {
      mode: OperationMode::Json,
      manual_config: None,
      json_config: Some(JsonModeConfig {
        json_output: "{ \"name\": \"{{name}}\", \"age\": {{age}} }".to_string(),
        use_expressions: true,
        validate_json: true,
        error_handling: JsonErrorHandling::StopExecution,
      }),
      options: EditFieldsOptions::default(),
    };
    assert!(valid_json_config.validate().is_ok());

    // 无效的 JSON 模式配置（空模板）
    let invalid_json_config = EditFieldsConfig {
      mode: OperationMode::Json,
      manual_config: None,
      json_config: Some(JsonModeConfig {
        json_output: "".to_string(),
        use_expressions: false,
        validate_json: true,
        error_handling: JsonErrorHandling::StopExecution,
      }),
      options: EditFieldsOptions::default(),
    };
    assert!(invalid_json_config.validate().is_err());
  }

  #[test]
  fn test_serialization() {
    // 测试枚举序列化
    let mode = OperationMode::Manual;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: OperationMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);

    let operation = OperationKind::Set;
    let serialized = serde_json::to_string(&operation).unwrap();
    let deserialized: OperationKind = serde_json::from_str(&serialized).unwrap();
    assert_eq!(operation, deserialized);

    let include_mode = IncludeMode::Selected;
    let serialized = serde_json::to_string(&include_mode).unwrap();
    let deserialized: IncludeMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(include_mode, deserialized);
  }

  #[test]
  fn test_node_creation() {
    let node = EditFieldsNode::new();
    assert!(node.is_ok());

    let node = node.unwrap();
    assert_eq!(node.default_version().major, 1);
    assert_eq!(node.node_executors().len(), 1);
  }
}
