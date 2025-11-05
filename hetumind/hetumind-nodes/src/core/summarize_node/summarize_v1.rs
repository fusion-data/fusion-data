use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, FlowNode, InputPortConfig, NodeDefinition,
    NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig, RegistrationError,
    make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  AggregateField, ErrorHandlingStrategy, GroupByConfig, OutputFormat, SerializationStyle, SummarizeConfig,
  utils::{aggregate_data, format_output},
};

/// Summarize 数据聚合节点 V1
///
/// 用于对数据进行聚合计算和统计，支持多种聚合操作、分组和输出格式。
///
/// # 聚合操作
/// - `Count`: 计数
/// - `Sum`: 求和
/// - `Avg`: 平均值
/// - `Min`: 最小值
/// - `Max`: 最大值
/// - `Median`: 中位数
/// - `StdDev`: 标准差
/// - `Variance`: 方差
/// - `Concat`: 连接字符串
/// - `Join`: 连接字符串（带分隔符）
/// - `CountUnique`: 唯一值计数
/// - `CountEmpty`: 空值计数
/// - `CountNotEmpty`: 非空值计数
/// - `First`: 第一个值
/// - `Last`: 最后一个值
///
/// # 分组支持
/// - 单字段分组
/// - 多种排序方式
/// - 可选择保留原始数据
///
/// # 输出格式
/// - JSON 对象格式
/// - 键值对数组
/// - 表格格式（对象数组）
///
/// # 序列化风格
/// - snake_case（默认）
/// - camelCase
/// - PascalCase
/// - kebab-case
#[derive(Debug)]
#[allow(dead_code)]
pub struct SummarizeV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl FlowNode for SummarizeV1 {
  #[allow(unused_variables)]
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Summarize 数据聚合节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Summarize 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::warn!("Summarize 节点没有接收到输入数据，返回空结果");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取聚合字段配置
    let aggregate_fields: Vec<AggregateField> = node.get_parameter("aggregate_fields")?;

    // 获取分组配置
    let group_by: Option<GroupByConfig> = node.get_optional_parameter("group_by");

    // 获取输出格式
    let output_format: OutputFormat = node.get_optional_parameter("output_format").unwrap_or(OutputFormat::Json);

    // 获取序列化风格
    let serialization_style: SerializationStyle =
      node.get_optional_parameter("serialization_style").unwrap_or(SerializationStyle::SnakeCase);

    // 获取其他选项
    let include_metadata: Option<bool> = node.get_optional_parameter("include_metadata");
    let error_handling: Option<ErrorHandlingStrategy> = node.get_optional_parameter("error_handling");

    let config = SummarizeConfig {
      aggregate_fields,
      group_by,
      output_format,
      serialization_style,
      include_metadata,
      error_handling,
    };

    // 验证配置
    if let Err(e) = config.validate() {
      log::error!("Summarize 配置验证失败: {}", e);
      return Err(NodeExecutionError::ConfigurationError(format!("Invalid Summarize configuration: {}", e)));
    }

    log::debug!(
      "聚合配置: 字段数={}, 分组={}, 输出格式={:?}",
      config.aggregate_fields.len(),
      config.group_by.is_some(),
      config.output_format
    );

    // 打印配置详情（调试模式）
    for (index, field) in config.aggregate_fields.iter().enumerate() {
      log::debug!(
        "聚合字段 {}: {} -> {} ({})",
        index,
        field.source_field,
        field.output_field,
        serde_json::to_string(&field.operation).unwrap_or_default()
      );
    }

    // 执行聚合操作
    let aggregated_data = aggregate_data(&input_items, &config, &context.current_node_name)?;

    // 格式化输出
    let formatted_output = format_output(&aggregated_data, &config)?;

    log::info!("Summarize 节点执行完成: 输入 {} 项，输出 {} 项", input_items.len(), formatted_output.len());

    let execution_data: Vec<ExecutionData> =
      formatted_output.into_iter().map(|value| ExecutionData::new_json(value, None)).collect();

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(execution_data)])]))
  }
}

impl TryFrom<NodeDefinition> for SummarizeV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Output"))
      .add_property(
        // 聚合字段配置
        NodeProperty::new(NodePropertyKind::FixedCollection)
          .with_display_name("聚合字段")
          .with_name("aggregate_fields")
          .with_required(true)
          .with_description("要聚合的字段配置列表")
          .with_placeholder("添加聚合字段..."),
      )
      .add_property(
        // 分组配置
        NodeProperty::new(NodePropertyKind::Json)
          .with_display_name("分组配置")
          .with_name("group_by")
          .with_required(false)
          .with_description("按字段分组进行聚合")
          .with_placeholder("配置分组..."),
      )
      .add_property(
        // 输出格式
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("输出格式")
          .with_name("output_format")
          .with_required(false)
          .with_description("选择输出数据的格式")
          .with_value(json!(OutputFormat::Json))
          .with_options(vec![
            Box::new(NodeProperty::new_option("JSON", "json", json!(OutputFormat::Json), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "键值对数组",
              "key_value_array",
              json!(OutputFormat::KeyValueArray),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "表格格式",
              "table_format",
              json!(OutputFormat::TableFormat),
              NodePropertyKind::String,
            )),
          ]),
      )
      .add_property(
        // 序列化风格
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("序列化风格")
          .with_name("serialization_style")
          .with_required(false)
          .with_description("字段名的序列化风格")
          .with_value(json!(SerializationStyle::SnakeCase))
          .with_options(vec![
            Box::new(NodeProperty::new_option(
              "snake_case",
              "snake_case",
              json!(SerializationStyle::SnakeCase),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "camelCase",
              "camel_case",
              json!(SerializationStyle::CamelCase),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "PascalCase",
              "pascal_case",
              json!(SerializationStyle::PascalCase),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "kebab-case",
              "kebab_case",
              json!(SerializationStyle::KebabCase),
              NodePropertyKind::String,
            )),
          ]),
      )
      .add_property(
        // 高级选项
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("高级选项")
          .with_name("advanced_options")
          .with_required(false)
          .with_description("高级配置选项"),
      )
      .add_property(
        // 包含元数据
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("包含元数据")
          .with_name("include_metadata")
          .with_required(false)
          .with_description("是否在输出中包含聚合元数据")
          .with_value(json!(false)),
      )
      .add_property(
        // 错误处理
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("错误处理")
          .with_name("error_handling")
          .with_required(false)
          .with_description("遇到错误时的处理策略")
          .with_value(json!(ErrorHandlingStrategy::SkipError))
          .with_options(vec![
            Box::new(NodeProperty::new_option(
              "跳过错误值",
              "skip_error",
              json!(ErrorHandlingStrategy::SkipError),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "使用默认值",
              "use_default",
              json!(ErrorHandlingStrategy::UseDefault),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "停止执行",
              "stop_execution",
              json!(ErrorHandlingStrategy::StopExecution),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "记录错误但继续",
              "log_and_continue",
              json!(ErrorHandlingStrategy::LogAndContinue),
              NodePropertyKind::String,
            )),
          ]),
      )
      .add_property(
        // 聚合字段详细配置（用于 FixedCollection）
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("字段配置")
          .with_name("field_config")
          .with_required(false)
          .with_description("聚合字段的详细配置"),
      )
      .add_property(
        // 分组字段详细配置
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("分组字段配置")
          .with_name("group_field_config")
          .with_required(false)
          .with_description("分组字段的详细配置"),
      );
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::{
    SummarizeNode,
    summarize_node::{AggregateOperation, DataType, GroupSortOrder},
  };
  use hetumind_core::workflow::Node;

  #[test]
  fn test_node_definition_properties() {
    let node = SummarizeNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    // 验证基本属性
    assert_eq!(definition.version, Version::new(1, 0, 0));
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);

    // 验证属性配置
    let aggregate_fields_prop = definition.properties.iter().find(|p| p.name == "aggregate_fields");
    assert!(aggregate_fields_prop.is_some());
    assert!(aggregate_fields_prop.unwrap().required);

    let output_format_prop = definition.properties.iter().find(|p| p.name == "output_format");
    assert!(output_format_prop.is_some());
    assert!(!output_format_prop.unwrap().required);

    let serialization_style_prop = definition.properties.iter().find(|p| p.name == "serialization_style");
    assert!(serialization_style_prop.is_some());
    assert!(!serialization_style_prop.unwrap().required);

    let group_by_prop = definition.properties.iter().find(|p| p.name == "group_by");
    assert!(group_by_prop.is_some());
    assert!(!group_by_prop.unwrap().required);
  }

  // Field name conversion tests are in utils.rs

  #[test]
  fn test_aggregate_operation_string_conversion() {
    // 测试操作类型的字符串表示
    let count_json = serde_json::to_string(&AggregateOperation::Count).unwrap();
    assert_eq!(count_json, "\"count\"");

    let sum_json = serde_json::to_string(&AggregateOperation::Sum).unwrap();
    assert_eq!(sum_json, "\"sum\"");

    let avg_json = serde_json::to_string(&AggregateOperation::Avg).unwrap();
    assert_eq!(avg_json, "\"avg\"");
  }

  #[test]
  fn test_serialization_style_string_conversion() {
    // 测试序列化风格的字符串表示
    let snake_json = serde_json::to_string(&SerializationStyle::SnakeCase).unwrap();
    assert_eq!(snake_json, "\"snake_case\"");

    let camel_json = serde_json::to_string(&SerializationStyle::CamelCase).unwrap();
    assert_eq!(camel_json, "\"camel_case\"");

    let pascal_json = serde_json::to_string(&SerializationStyle::PascalCase).unwrap();
    assert_eq!(pascal_json, "\"pascal_case\"");

    let kebab_json = serde_json::to_string(&SerializationStyle::KebabCase).unwrap();
    assert_eq!(kebab_json, "\"kebab_case\"");
  }

  #[test]
  fn test_data_type_string_conversion() {
    // 测试数据类型的字符串表示
    let string_json = serde_json::to_string(&DataType::String).unwrap();
    assert_eq!(string_json, "\"string\"");

    let number_json = serde_json::to_string(&DataType::Number).unwrap();
    assert_eq!(number_json, "\"number\"");

    let boolean_json = serde_json::to_string(&DataType::Boolean).unwrap();
    assert_eq!(boolean_json, "\"boolean\"");
  }

  #[test]
  fn test_error_handling_strategy_string_conversion() {
    // 测试错误处理策略的字符串表示
    let skip_json = serde_json::to_string(&ErrorHandlingStrategy::SkipError).unwrap();
    assert_eq!(skip_json, "\"skip_error\"");

    let use_default_json = serde_json::to_string(&ErrorHandlingStrategy::UseDefault).unwrap();
    assert_eq!(use_default_json, "\"use_default\"");

    let stop_json = serde_json::to_string(&ErrorHandlingStrategy::StopExecution).unwrap();
    assert_eq!(stop_json, "\"stop_execution\"");

    let log_json = serde_json::to_string(&ErrorHandlingStrategy::LogAndContinue).unwrap();
    assert_eq!(log_json, "\"log_and_continue\"");
  }

  #[test]
  fn test_group_sort_order_string_conversion() {
    // 测试分组排序方式的字符串表示
    let group_asc_json = serde_json::to_string(&GroupSortOrder::GroupAsc).unwrap();
    assert_eq!(group_asc_json, "\"group_asc\"");

    let count_desc_json = serde_json::to_string(&GroupSortOrder::CountDesc).unwrap();
    assert_eq!(count_desc_json, "\"count_desc\"");

    let none_json = serde_json::to_string(&GroupSortOrder::None).unwrap();
    assert_eq!(none_json, "\"none\"");
  }

  #[test]
  fn test_output_format_string_conversion() {
    // 测试输出格式的字符串表示
    let json_json = serde_json::to_string(&OutputFormat::Json).unwrap();
    assert_eq!(json_json, "\"json\"");

    let key_value_json = serde_json::to_string(&OutputFormat::KeyValueArray).unwrap();
    assert_eq!(key_value_json, "\"key_value_array\"");

    let table_json = serde_json::to_string(&OutputFormat::TableFormat).unwrap();
    assert_eq!(table_json, "\"table_format\"");
  }
}
