use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind,
    NodePropertyKindOptions, OutputPortConfig, RegistrationError, ValidationError, make_execution_data_map,
  },
};
use serde_json::{Value, json};

use super::{
  AggregateConfig, AggregateMode, AggregateOptions, FieldToAggregate,
  utils::{
    FieldExistenceTracker, add_binaries_to_item, apply_field_filter, get_field_value, prepare_fields_array,
    process_field_value,
  },
};

#[derive(Debug)]
pub struct AggregateV1 {
  pub definition: Arc<NodeDefinition>,
}

impl AggregateV1 {
  /// 执行 Individual Fields 聚合模式
  async fn execute_individual_fields(
    &self,
    context: &NodeExecutionContext,
    input_items: &[ExecutionData],
    config: &AggregateConfig,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    log::info!("执行 Individual Fields 聚合模式");

    let mut field_existence_tracker = FieldExistenceTracker::new();
    let mut aggregated_values: serde_json::Map<String, Value> = serde_json::Map::new();

    // 为每个字段进行聚合
    for field_config in &config.fields_to_aggregate {
      let field_path = &field_config.field_to_aggregate;
      let output_field_name = field_config.get_output_field_name();

      log::info!("聚合字段: {} -> {}", field_path, output_field_name);

      let mut field_values: Vec<Value> = Vec::new();

      // 遍历所有输入项
      for (item_index, item) in input_items.iter().enumerate() {
        let field_value = get_field_value(item.json(), field_path, config.options.disable_dot_notation);

        // 记录字段存在性
        field_existence_tracker.record_field_existence(field_path, field_value.is_some());

        // 处理字段值
        if let Some(processed_value) =
          process_field_value(field_value, config.options.keep_missing, config.options.merge_lists)
        {
          if let Value::Array(values) = processed_value {
            field_values.extend(values);
          } else {
            field_values.push(processed_value);
          }
        } else {
          log::debug!("跳过空值: 项 {}, 字段 {}", item_index, field_path);
        }
      }

      // 设置聚合后的值
      aggregated_values.insert(output_field_name, Value::Array(field_values));
    }

    // 创建输出项
    let mut new_item = ExecutionData::new_json(Value::Object(aggregated_values), None);

    // Note: ExecutionData 目前不支持 metadata，pairedItem 概念需要通过其他方式实现
    // 这里暂时注释掉，等待后续 ExecutionData 结构支持 metadata

    // 处理二进制数据（如果启用）
    if config.options.include_binaries {
      log::info!("包含二进制数据聚合");
      add_binaries_to_item(&mut new_item, input_items, config.options.keep_only_unique)?;
    }

    // 生成执行提示
    self.generate_field_hints(&field_existence_tracker)?;

    log::info!("Individual Fields 聚合完成");

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![new_item])])]))
  }

  /// 执行 All Item Data 聚合模式
  async fn execute_all_item_data(
    &self,
    context: &NodeExecutionContext,
    input_items: &[ExecutionData],
    config: &AggregateConfig,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    log::info!("执行 All Item Data 聚合模式");

    let destination_field = config.destination_field_name.as_ref().ok_or_else(|| {
      NodeExecutionError::ParameterValidation(ValidationError::required_field_missing("destination_field_name"))
    })?;

    // 准备字段过滤配置
    let fields_to_exclude = &config.fields_to_exclude;
    let fields_to_include = &config.fields_to_include;

    log::info!("目标字段: {}, 排除字段: {:?}, 包含字段: {:?}", destination_field, fields_to_exclude, fields_to_include);

    let mut filtered_items = Vec::new();
    let mut paired_items = Vec::new();

    // 处理每个输入项
    for (index, item) in input_items.iter().enumerate() {
      // 应用字段过滤
      if let Some(filtered_item) = apply_field_filter(item.json(), fields_to_exclude, fields_to_include) {
        filtered_items.push(filtered_item);
        paired_items.push(json!({ "item": index }));
      } else {
        log::debug!("跳过空项: {}", index);
      }
    }

    // 创建输出数据
    let output_json = json!({ destination_field: filtered_items });
    let mut new_item = ExecutionData::new_json(output_json, None);

    // Note: ExecutionData 目前不支持 metadata，pairedItem 概念需要通过其他方式实现

    // 处理二进制数据（如果启用）
    if config.options.include_binaries && !filtered_items.is_empty() {
      log::info!("包含二进制数据聚合");

      // 获取原始的输入项（用于二进制数据聚合）
      let original_items: Vec<ExecutionData> = paired_items
        .iter()
        .filter_map(|paired| {
          paired.get("item").and_then(|v| v.as_u64()).and_then(|idx| input_items.get(idx as usize)).cloned()
        })
        .collect();

      add_binaries_to_item(&mut new_item, &original_items, config.options.keep_only_unique)?;
    }

    log::info!("All Item Data 聚合完成 - 输入项: {}, 过滤后项: {}", input_items.len(), filtered_items.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![new_item])])]))
  }

  /// 生成字段存在性提示
  fn generate_field_hints(&self, tracker: &FieldExistenceTracker) -> Result<(), NodeExecutionError> {
    let completely_missing = tracker.get_completely_missing_fields();
    let partially_missing = tracker.get_partially_missing_fields();

    // 为完全不存在的字段生成提示
    for field in completely_missing {
      log::warn!("字段 '{}' 在所有输入项中都未找到", field);
      // 这里可以添加执行提示机制
    }

    // 为部分存在的字段生成信息
    for field in partially_missing {
      log::info!("字段 '{}' 在部分输入项中未找到", field);
    }

    Ok(())
  }
}

/// Aggregate 数据聚合节点 V1
///
/// 用于将多个数据项的字段合并成单个数据项中的列表。
/// 支持两种聚合模式：
/// 1. Individual Fields: 选择性聚合指定字段
/// 2. All Item Data: 将所有数据项聚合成单个列表
///
/// # 功能特性
/// - 支持点记号字段路径
/// - 字段重命名
/// - 列表合并选项
/// - 二进制数据处理
/// - 空值过滤
/// - 字段存在性提示
#[async_trait]
impl NodeExecutable for AggregateV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 Aggregate 聚合节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Aggregate 节点接收到 {} 个输入项", input_data.len());
      for (i, item) in input_data.iter().enumerate() {
        log::debug!("输入项 {}: {}", i, serde_json::to_string(item.json()).unwrap_or_default());
      }
      input_data
    } else {
      log::error!("Aggregate 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]));
    };

    // 获取聚合配置
    let aggregate_mode: AggregateMode = node.get_parameter("aggregate")?;
    let options: AggregateOptions = node.get_optional_parameter("options").unwrap_or_default();

    log::info!("聚合模式: {:?}, 选项: {:?}", aggregate_mode, options);

    match aggregate_mode {
      AggregateMode::AggregateIndividualFields => {
        // 获取字段聚合配置
        let fields_to_aggregate: Vec<FieldToAggregate> = node.get_parameter("fields_to_aggregate")?;

        if fields_to_aggregate.is_empty() {
          return Err(NodeExecutionError::ParameterValidation(ValidationError::required_field_missing(
            "fields_to_aggregate",
          )));
        }

        let config = AggregateConfig {
          aggregate: aggregate_mode,
          fields_to_aggregate,
          destination_field_name: None,
          fields_to_exclude: vec![],
          fields_to_include: vec![],
          options,
        };

        // 验证配置
        config.validate().map_err(|e| {
          NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
            "aggregation_config",
            format!("Invalid configuration: {}", e),
          ))
        })?;

        self.execute_individual_fields(context, &input_items, &config).await
      }
      AggregateMode::AggregateAllItemData => {
        // 获取目标字段名
        let destination_field_name: String = node.get_parameter("destination_field_name")?;

        // 获取字段过滤配置
        let fields_to_exclude_str: String = node.get_optional_parameter("fields_to_exclude").unwrap_or_default();
        let fields_to_include_str: String = node.get_optional_parameter("fields_to_include").unwrap_or_default();

        let fields_to_exclude = prepare_fields_array(&fields_to_exclude_str, "Fields To Exclude");
        let fields_to_include = prepare_fields_array(&fields_to_include_str, "Fields To Include");

        let config = AggregateConfig {
          aggregate: aggregate_mode,
          fields_to_aggregate: vec![],
          destination_field_name: Some(destination_field_name),
          fields_to_exclude,
          fields_to_include,
          options,
        };

        // 验证配置
        config.validate().map_err(|e| {
          NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
            "aggregation_config",
            format!("Invalid configuration: {}", e),
          ))
        })?;

        self.execute_all_item_data(context, &input_items, &config).await
      }
    }
  }
}

impl TryFrom<NodeDefinition> for AggregateV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .add_input(InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build())
      .add_output(OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build())
      .add_property(
        // 聚合模式选择
        NodeProperty::builder()
          .display_name("Aggregate".to_string())
          .name("aggregate")
          .kind(NodePropertyKind::Options)
          .required(true)
          .description("Choose the aggregation mode".to_string())
          .value(json!("aggregateIndividualFields"))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "Individual Fields",
              "aggregateIndividualFields",
              json!("aggregateIndividualFields"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "All Item Data (Into a Single List)",
              "aggregateAllItemData",
              json!("aggregateAllItemData"),
              NodePropertyKind::String,
            )),
          ])
          .build(),
      )
      .add_property(
        // Individual Fields 模式的字段配置
        NodeProperty::builder()
          .display_name("Fields To Aggregate".to_string())
          .name("fields_to_aggregate")
          .required(false)
          .description("Fields to aggregate together".to_string())
          .placeholder("Add Field To Aggregate".to_string())
          .kind(NodePropertyKind::FixedCollection)
          .kind_options(NodePropertyKindOptions::builder().multiple_values(true).build())
          .build(),
      )
      .add_property(
        // All Item Data 模式的目标字段配置
        NodeProperty::builder()
          .display_name("Destination Field Name".to_string())
          .name("destination_field_name")
          .required(false)
          .description("The name of the field to put the aggregated data in".to_string())
          .placeholder("e.g. items".to_string())
          .kind(NodePropertyKind::String)
          .build(),
      )
      .add_property(
        // 字段排除配置
        NodeProperty::builder()
          .display_name("Fields To Exclude".to_string())
          .name("fields_to_exclude")
          .required(false)
          .description("Fields to exclude from aggregation".to_string())
          .placeholder("e.g. password,secret".to_string())
          .kind(NodePropertyKind::String)
          .build(),
      )
      .add_property(
        // 字段包含配置
        NodeProperty::builder()
          .display_name("Fields To Include".to_string())
          .name("fields_to_include")
          .required(false)
          .description("Fields to include in aggregation (if empty, include all)".to_string())
          .placeholder("e.g. name,email".to_string())
          .kind(NodePropertyKind::String)
          .build(),
      )
      .add_property(
        // 高级选项
        NodeProperty::builder()
          .display_name("Options".to_string())
          .name("options")
          .required(false)
          .description("Advanced aggregation options".to_string())
          .placeholder("Add Option".to_string())
          .options(vec![
            Box::new(NodeProperty::new_option(
              "Disable Dot Notation",
              "disable_dot_notation",
              json!(false),
              NodePropertyKind::Boolean,
            )),
            Box::new(NodeProperty::new_option("Merge Lists", "merge_lists", json!(false), NodePropertyKind::Boolean)),
            Box::new(NodeProperty::new_option("Keep Missing", "keep_missing", json!(true), NodePropertyKind::Boolean)),
            Box::new(NodeProperty::new_option(
              "Include Binaries",
              "include_binaries",
              json!(false),
              NodePropertyKind::Boolean,
            )),
            Box::new(NodeProperty::new_option(
              "Keep Only Unique",
              "keep_only_unique",
              json!(false),
              NodePropertyKind::Boolean,
            )),
          ])
          .build(),
      );

    Ok(Self { definition: Arc::new(definition) })
  }
}
