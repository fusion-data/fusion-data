//! Summarize 数据聚合节点实现
//!
//! 参考 n8n 的 Summarize 节点设计，用于对数据进行聚合计算和统计。
//! 支持多种聚合操作、分组、输出格式和序列化风格。

use std::sync::Arc;

use fusion_common::ahash::HashSet;
use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError, ValidationError},
};
use serde::{Deserialize, Serialize};

mod summarize_v1;
mod utils;

use summarize_v1::SummarizeV1;

use crate::constants::SUMMARIZE_NODE_KIND;

/// 聚合操作类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateOperation {
  /// 计数
  Count,
  /// 求和
  Sum,
  /// 平均值
  Avg,
  /// 最小值
  Min,
  /// 最大值
  Max,
  /// 中位数
  Median,
  /// 标准差
  StdDev,
  /// 方差
  Variance,
  /// 连接字符串
  Concat,
  /// 连接字符串（带分隔符）
  Join,
  /// 唯一值计数
  CountUnique,
  /// 空值计数
  CountEmpty,
  /// 非空值计数
  CountNotEmpty,
  /// 第一个值
  First,
  /// 最后一个值
  Last,
}

/// 序列化风格
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerializationStyle {
  /// snake_case (默认)
  SnakeCase,
  /// camelCase
  CamelCase,
  /// PascalCase
  PascalCase,
  /// kebab-case
  KebabCase,
}

/// 输出格式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
  /// JSON 对象格式
  Json,
  /// 键值对数组
  KeyValueArray,
  /// 表格格式（对象数组）
  TableFormat,
}

/// 聚合字段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateField {
  /// 源字段名（支持 JSON 路径）
  pub source_field: String,
  /// 输出字段名
  pub output_field: String,
  /// 聚合操作
  pub operation: AggregateOperation,
  /// 分隔符（用于 Join 操作）
  pub separator: Option<String>,
  /// 是否忽略空值
  pub ignore_empty: Option<bool>,
  /// 数据类型（用于类型转换）
  pub data_type: Option<DataType>,
  /// 自定义格式化
  pub format: Option<String>,
}

/// 数据类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
  /// 字符串
  String,
  /// 数字
  Number,
  /// 布尔值
  Boolean,
  /// 日期
  Date,
  /// 数组
  Array,
  /// 对象
  Object,
}

/// 分组配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupByConfig {
  /// 分组字段（支持 JSON 路径）
  pub group_field: String,
  /// 分组字段输出名称
  pub group_output_name: String,
  /// 是否保留原始数据
  pub keep_original_data: Option<bool>,
  /// 排序方式
  pub sort_by: Option<GroupSortOrder>,
}

/// 分组排序方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupSortOrder {
  /// 按分组值升序
  GroupAsc,
  /// 按分组值降序
  GroupDesc,
  /// 按计数升序
  CountAsc,
  /// 按计数降序
  CountDesc,
  /// 不排序
  None,
}

/// Summarize 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeConfig {
  /// 要聚合的字段列表
  pub aggregate_fields: Vec<AggregateField>,
  /// 分组配置（可选）
  pub group_by: Option<GroupByConfig>,
  /// 输出格式
  pub output_format: OutputFormat,
  /// 序列化风格
  pub serialization_style: SerializationStyle,
  /// 是否包含元数据
  pub include_metadata: Option<bool>,
  /// 错误处理策略
  pub error_handling: Option<ErrorHandlingStrategy>,
}

/// 错误处理策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandlingStrategy {
  /// 跳过错误值
  SkipError,
  /// 使用默认值
  UseDefault,
  /// 停止执行
  StopExecution,
  /// 记录错误但继续
  LogAndContinue,
}

impl AggregateField {
  /// 验证聚合字段配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.source_field.trim().is_empty() {
      return Err(ValidationError::invalid_field_value(
        "source_field".to_string(),
        "Source field cannot be empty".to_string(),
      ));
    }

    if self.output_field.trim().is_empty() {
      return Err(ValidationError::invalid_field_value(
        "output_field".to_string(),
        "Output field cannot be empty".to_string(),
      ));
    }

    // Join 操作需要分隔符
    if self.operation == AggregateOperation::Join && self.separator.as_ref().is_none_or(|s| s.is_empty()) {
      return Err(ValidationError::invalid_field_value(
        "separator".to_string(),
        "Separator is required for Join operation".to_string(),
      ));
    }

    // 数值操作的数据类型验证
    if matches!(
      self.operation,
      AggregateOperation::Sum
        | AggregateOperation::Avg
        | AggregateOperation::Min
        | AggregateOperation::Max
        | AggregateOperation::Median
        | AggregateOperation::StdDev
        | AggregateOperation::Variance
    ) && let Some(ref data_type) = self.data_type
      && !matches!(data_type, DataType::Number | DataType::String | DataType::Date)
    {
      return Err(ValidationError::invalid_field_value(
        "data_type".to_string(),
        "Numeric operations require Number, String, or Date data type".to_string(),
      ));
    }

    Ok(())
  }

  /// 获取有效的分隔符
  pub fn get_separator(&self) -> String {
    self.separator.clone().unwrap_or_else(|| match self.operation {
      AggregateOperation::Join => ", ".to_string(),
      AggregateOperation::Concat => "".to_string(),
      _ => "".to_string(),
    })
  }

  /// 是否应该忽略空值
  pub fn should_ignore_empty(&self) -> bool {
    self.ignore_empty.unwrap_or(true)
  }
}

impl GroupByConfig {
  /// 验证分组配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.group_field.trim().is_empty() {
      return Err(ValidationError::invalid_field_value(
        "group_field".to_string(),
        "Group field cannot be empty".to_string(),
      ));
    }

    if self.group_output_name.trim().is_empty() {
      return Err(ValidationError::invalid_field_value(
        "group_output_name".to_string(),
        "Group output name cannot be empty".to_string(),
      ));
    }

    Ok(())
  }

  /// 获取分组字段输出名称
  pub fn get_group_output_name(&self) -> String {
    if self.group_output_name.trim().is_empty() { self.group_field.clone() } else { self.group_output_name.clone() }
  }

  /// 是否保留原始数据
  pub fn should_keep_original_data(&self) -> bool {
    self.keep_original_data.unwrap_or(false)
  }
}

impl SummarizeConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.aggregate_fields.is_empty() {
      return Err(ValidationError::invalid_field_value(
        "aggregate_fields".to_string(),
        "At least one aggregate field is required".to_string(),
      ));
    }

    // 验证聚合字段
    for (index, field) in self.aggregate_fields.iter().enumerate() {
      field.validate().map_err(|e| {
        ValidationError::invalid_field_value(
          format!("aggregate_fields[{}]", index),
          format!("Invalid aggregate field: {}", e),
        )
      })?;
    }

    // 验证分组配置
    if let Some(ref group_by) = self.group_by {
      group_by.validate().map_err(|e| {
        ValidationError::invalid_field_value("group_by".to_string(), format!("Invalid group configuration: {}", e))
      })?;
    }

    // 验证字段名唯一性
    let mut field_names = HashSet::default();
    for field in &self.aggregate_fields {
      if !field_names.insert(&field.output_field) {
        return Err(ValidationError::invalid_field_value(
          "aggregate_fields".to_string(),
          format!("Duplicate output field name: {}", field.output_field),
        ));
      }
    }

    Ok(())
  }

  /// 获取有效的序列化风格
  #[allow(dead_code)]
  pub fn get_serialization_style(&self) -> SerializationStyle {
    self.serialization_style.clone()
  }

  /// 是否包含元数据
  pub fn should_include_metadata(&self) -> bool {
    self.include_metadata.unwrap_or(false)
  }

  /// 获取错误处理策略
  pub fn get_error_handling(&self) -> ErrorHandlingStrategy {
    self.error_handling.clone().unwrap_or(ErrorHandlingStrategy::SkipError)
  }
}

impl Default for AggregateOperation {
  fn default() -> Self {
    Self::Count
  }
}

impl Default for SerializationStyle {
  fn default() -> Self {
    Self::SnakeCase
  }
}

impl Default for OutputFormat {
  fn default() -> Self {
    Self::Json
  }
}

impl Default for DataType {
  fn default() -> Self {
    Self::String
  }
}

impl Default for GroupSortOrder {
  fn default() -> Self {
    Self::None
  }
}

impl Default for ErrorHandlingStrategy {
  fn default() -> Self {
    Self::SkipError
  }
}

impl Default for SummarizeConfig {
  fn default() -> Self {
    Self {
      aggregate_fields: vec![AggregateField {
        source_field: "{{ $json }}".to_string(),
        output_field: "count".to_string(),
        operation: AggregateOperation::Count,
        separator: None,
        ignore_empty: Some(true),
        data_type: None,
        format: None,
      }],
      group_by: None,
      output_format: OutputFormat::Json,
      serialization_style: SerializationStyle::SnakeCase,
      include_metadata: Some(false),
      error_handling: Some(ErrorHandlingStrategy::SkipError),
    }
  }
}

pub struct SummarizeNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl SummarizeNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(SummarizeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(SUMMARIZE_NODE_KIND, "Summarize")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .with_description("对数据进行聚合计算和统计。支持多种聚合操作、分组和输出格式。")
      .with_icon("calculator")
  }
}

impl Node for SummarizeNode {
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

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = SummarizeNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::Summarize");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input]);
    assert_eq!(&definition.display_name, "Summarize");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = SummarizeNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_aggregate_operation_equality() {
    assert_eq!(AggregateOperation::Count, AggregateOperation::Count);
    assert_ne!(AggregateOperation::Count, AggregateOperation::Sum);

    // 测试序列化和反序列化
    let operation = AggregateOperation::Avg;
    let serialized = serde_json::to_string(&operation).unwrap();
    let deserialized: AggregateOperation = serde_json::from_str(&serialized).unwrap();
    assert_eq!(operation, deserialized);
  }

  #[test]
  fn test_serialization_style_equality() {
    assert_eq!(SerializationStyle::SnakeCase, SerializationStyle::SnakeCase);
    assert_ne!(SerializationStyle::SnakeCase, SerializationStyle::CamelCase);

    // 测试序列化和反序列化
    let style = SerializationStyle::CamelCase;
    let serialized = serde_json::to_string(&style).unwrap();
    let deserialized: SerializationStyle = serde_json::from_str(&serialized).unwrap();
    assert_eq!(style, deserialized);
  }

  #[test]
  fn test_aggregate_field_validation() {
    // 有效的字段配置
    let valid_field = AggregateField {
      source_field: "price".to_string(),
      output_field: "total_price".to_string(),
      operation: AggregateOperation::Sum,
      separator: None,
      ignore_empty: Some(true),
      data_type: Some(DataType::Number),
      format: None,
    };
    assert!(valid_field.validate().is_ok());

    // 无效的字段配置（空源字段）
    let invalid_field = AggregateField {
      source_field: "".to_string(),
      output_field: "output".to_string(),
      operation: AggregateOperation::Count,
      separator: None,
      ignore_empty: None,
      data_type: None,
      format: None,
    };
    assert!(invalid_field.validate().is_err());

    // Join 操作需要分隔符
    let join_without_separator = AggregateField {
      source_field: "tags".to_string(),
      output_field: "joined_tags".to_string(),
      operation: AggregateOperation::Join,
      separator: None,
      ignore_empty: None,
      data_type: None,
      format: None,
    };
    assert!(join_without_separator.validate().is_err());
  }

  #[test]
  fn test_group_by_config_validation() {
    // 有效的分组配置
    let valid_group = GroupByConfig {
      group_field: "category".to_string(),
      group_output_name: "group_name".to_string(),
      keep_original_data: Some(false),
      sort_by: Some(GroupSortOrder::CountDesc),
    };
    assert!(valid_group.validate().is_ok());

    // 无效的分组配置（空分组字段）
    let invalid_group = GroupByConfig {
      group_field: "".to_string(),
      group_output_name: "group".to_string(),
      keep_original_data: None,
      sort_by: None,
    };
    assert!(invalid_group.validate().is_err());
  }

  #[test]
  fn test_summarize_config_validation() {
    // 有效的配置
    let valid_config = SummarizeConfig {
      aggregate_fields: vec![
        AggregateField {
          source_field: "amount".to_string(),
          output_field: "total_amount".to_string(),
          operation: AggregateOperation::Sum,
          separator: None,
          ignore_empty: Some(true),
          data_type: Some(DataType::Number),
          format: None,
        },
        AggregateField {
          source_field: "name".to_string(),
          output_field: "count".to_string(),
          operation: AggregateOperation::Count,
          separator: None,
          ignore_empty: Some(true),
          data_type: None,
          format: None,
        },
      ],
      group_by: Some(GroupByConfig {
        group_field: "category".to_string(),
        group_output_name: "category".to_string(),
        keep_original_data: None,
        sort_by: None,
      }),
      output_format: OutputFormat::Json,
      serialization_style: SerializationStyle::SnakeCase,
      include_metadata: Some(false),
      error_handling: Some(ErrorHandlingStrategy::SkipError),
    };
    assert!(valid_config.validate().is_ok());

    // 无效的配置（没有聚合字段）
    let invalid_config = SummarizeConfig {
      aggregate_fields: vec![],
      group_by: None,
      output_format: OutputFormat::Json,
      serialization_style: SerializationStyle::SnakeCase,
      include_metadata: None,
      error_handling: None,
    };
    assert!(invalid_config.validate().is_err());

    // 重复的字段名
    let duplicate_fields_config = SummarizeConfig {
      aggregate_fields: vec![
        AggregateField {
          source_field: "field1".to_string(),
          output_field: "result".to_string(),
          operation: AggregateOperation::Count,
          separator: None,
          ignore_empty: None,
          data_type: None,
          format: None,
        },
        AggregateField {
          source_field: "field2".to_string(),
          output_field: "result".to_string(),
          operation: AggregateOperation::Sum,
          separator: None,
          ignore_empty: None,
          data_type: None,
          format: None,
        },
      ],
      group_by: None,
      output_format: OutputFormat::Json,
      serialization_style: SerializationStyle::SnakeCase,
      include_metadata: None,
      error_handling: None,
    };
    assert!(duplicate_fields_config.validate().is_err());
  }

  #[test]
  fn test_default_config() {
    let default_config = SummarizeConfig::default();
    assert_eq!(default_config.aggregate_fields.len(), 1);
    assert_eq!(default_config.aggregate_fields[0].operation, AggregateOperation::Count);
    assert_eq!(default_config.output_format, OutputFormat::Json);
    assert_eq!(default_config.serialization_style, SerializationStyle::SnakeCase);
    assert_eq!(default_config.include_metadata, Some(false));
    assert_eq!(default_config.error_handling, Some(ErrorHandlingStrategy::SkipError));
  }

  #[test]
  fn test_aggregate_field_helper_methods() {
    let field = AggregateField {
      source_field: "tags".to_string(),
      output_field: "joined_tags".to_string(),
      operation: AggregateOperation::Join,
      separator: Some(" | ".to_string()),
      ignore_empty: Some(false),
      data_type: None,
      format: None,
    };

    assert_eq!(field.get_separator(), " | ");
    assert!(!field.should_ignore_empty());

    let field_with_default_separator = AggregateField {
      source_field: "words".to_string(),
      output_field: "sentence".to_string(),
      operation: AggregateOperation::Join,
      separator: None,
      ignore_empty: None,
      data_type: None,
      format: None,
    };

    assert_eq!(field_with_default_separator.get_separator(), ", ");
    assert!(field_with_default_separator.should_ignore_empty());
  }

  #[test]
  fn test_group_by_helper_methods() {
    let group = GroupByConfig {
      group_field: "category".to_string(),
      group_output_name: "group_name".to_string(),
      keep_original_data: Some(true),
      sort_by: Some(GroupSortOrder::CountDesc),
    };

    assert_eq!(group.get_group_output_name(), "group_name");
    assert!(group.should_keep_original_data());

    let group_with_empty_output_name = GroupByConfig {
      group_field: "type".to_string(),
      group_output_name: "".to_string(),
      keep_original_data: None,
      sort_by: None,
    };

    assert_eq!(group_with_empty_output_name.get_group_output_name(), "type");
    assert!(!group_with_empty_output_name.should_keep_original_data());
  }
}
