use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, ValidationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  FieldToSplit, IncludeStrategy, SplitOutConfig,
  utils::{self, prepare_fields_array},
};

#[derive(Debug)]
pub struct SplitOutV1 {
  pub definition: Arc<NodeDefinition>,
}

impl SplitOutV1 {
  /// 执行数据拆分
  async fn execute_split_out(
    &self,
    context: &NodeExecutionContext,
    input_items: &[ExecutionData],
    config: &SplitOutConfig,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 Split Out 节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    log::info!("Split Out 节点接收到 {} 个输入项", input_items.len());

    let mut all_output_items = Vec::new();
    let mut missing_fields_tracker = utils::MissingFieldsTracker::new();

    // 处理每个输入项
    for (item_index, input_item) in input_items.iter().enumerate() {
      log::debug!("处理输入项 {}: {}", item_index, serde_json::to_string(input_item.json()).unwrap_or_default());

      let split_results = self.process_input_item(input_item, config, &mut missing_fields_tracker)?;
      let split_results_count = split_results.len();
      all_output_items.extend(split_results);

      log::info!("输入项 {} 拆分完成，产生 {} 个输出项", item_index, split_results_count);
    }

    // 生成执行提示
    self.generate_execution_hints(&missing_fields_tracker)?;

    log::info!("Split Out 节点执行完成 - 总输入项: {}, 总输出项: {}", input_items.len(), all_output_items.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(all_output_items)])]))
  }

  /// 处理单个输入项
  fn process_input_item(
    &self,
    input_item: &ExecutionData,
    config: &SplitOutConfig,
    missing_fields_tracker: &mut utils::MissingFieldsTracker,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let item_json = input_item.json();
    let mut split_results = Vec::new();

    // 为每个要拆分的字段处理数据
    for (field_index, field_config) in config.fields_to_split.iter().enumerate() {
      let field_path = &field_config.field_to_split;
      let destination_field = field_config.get_destination_field();

      log::debug!("处理字段 {} -> {}", field_path, destination_field);

      // 提取要拆分的数据
      let field_data = utils::extract_field_data(item_json, field_path, config.disable_dot_notation);

      match field_data {
        Some(data) => {
          // 标记字段存在
          missing_fields_tracker.record_field_existence(field_path, true);

          // 标准化数据为可拆分格式
          let normalized_data = utils::normalize_data_to_split(data);

          if normalized_data.is_empty() {
            log::debug!("字段 {} 的数据为空，跳过拆分", field_path);
            continue;
          }

          // 创建拆分后的数据项
          let field_results =
            self.create_split_items(&normalized_data, item_json, &destination_field, config, field_index)?;

          split_results.extend(field_results);
        }
        None => {
          // 标记字段不存在
          missing_fields_tracker.record_field_existence(field_path, false);
          log::warn!("字段 '{}' 在输入项中未找到", field_path);
        }
      }
    }

    // 如果没有拆分出任何数据，创建空项
    if split_results.is_empty() {
      log::info!("没有拆分出任何数据，创建空输出项");
      split_results.push(ExecutionData::new_json(json!({}), None));
    }

    Ok(split_results)
  }

  /// 创建拆分后的数据项
  fn create_split_items(
    &self,
    split_data: &[JsonValue],
    original_item: &JsonValue,
    destination_field: &str,
    config: &SplitOutConfig,
    field_index: usize,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let mut results = Vec::new();

    for (element_index, element) in split_data.iter().enumerate() {
      let mut new_item_json = json!({});

      // 应用字段映射
      new_item_json[destination_field] = element.clone();

      // 根据包含策略处理其他字段
      match config.include_strategy {
        IncludeStrategy::NoOtherFields => {
          // 仅保留拆分字段
          // new_item_json 已经只包含目标字段
        }
        IncludeStrategy::AllOtherFields => {
          // 保留所有其他字段，但移除拆分的源字段
          self.apply_all_other_fields(
            &mut new_item_json,
            original_item,
            &config.fields_to_split[field_index].field_to_split,
          )?;
        }
        IncludeStrategy::SelectedOtherFields => {
          // 选择性保留字段
          self.apply_selected_fields(&mut new_item_json, original_item, &config.fields_to_include)?;
        }
      }

      // 处理二进制数据
      if config.include_binary {
        self.include_binary_data(&mut new_item_json, original_item)?;
      }

      // 创建执行数据项
      let execution_data = ExecutionData::new_json(new_item_json, None);
      results.push(execution_data);

      log::debug!("创建拆分项 {} - 字段索引: {}, 元素索引: {}", results.len(), field_index, element_index);
    }

    Ok(results)
  }

  /// 应用 "All Other Fields" 策略
  fn apply_all_other_fields(
    &self,
    new_item: &mut JsonValue,
    original_item: &JsonValue,
    field_to_remove: &str,
  ) -> Result<(), NodeExecutionError> {
    if let (Some(new_obj), Some(orig_obj)) = (new_item.as_object_mut(), original_item.as_object()) {
      // 复制所有原始字段
      for (key, value) in orig_obj {
        if key != field_to_remove {
          new_obj.insert(key.clone(), value.clone());
        }
      }
    }
    Ok(())
  }

  /// 应用 "Selected Fields" 策略
  fn apply_selected_fields(
    &self,
    new_item: &mut JsonValue,
    original_item: &JsonValue,
    fields_to_include: &[String],
  ) -> Result<(), NodeExecutionError> {
    if let (Some(new_obj), Some(orig_obj)) = (new_item.as_object_mut(), original_item.as_object()) {
      // 只复制选中的字段
      for field in fields_to_include {
        if let Some(value) = orig_obj.get(field) {
          new_obj.insert(field.clone(), value.clone());
        }
      }
    }
    Ok(())
  }

  /// 包含二进制数据
  fn include_binary_data(
    &self,
    _new_item: &mut JsonValue,
    original_item: &JsonValue,
  ) -> Result<(), NodeExecutionError> {
    // 如果原始项包含二进制数据，则包含到新项中
    // 注意：当前 ExecutionData 结构可能不支持多二进制数据，这里为未来扩展预留
    if let Some(_binary) = original_item.get("$binary") {
      log::debug!("包含二进制数据处理（当前实现为占位符）");
      // TODO: 实现二进制数据处理逻辑
    }
    Ok(())
  }

  /// 生成执行提示
  fn generate_execution_hints(&self, tracker: &utils::MissingFieldsTracker) -> Result<(), NodeExecutionError> {
    let missing_fields = tracker.get_completely_missing_fields();

    if !missing_fields.is_empty() {
      let hints: Vec<String> =
        missing_fields.iter().map(|field| format!("字段 '{}' 在所有输入项中都未找到", field)).collect();

      log::warn!("Split Out 执行提示: {}", hints.join(", "));

      // TODO: 添加到执行面板提示系统
      // self.add_execution_hints(hints.into_iter().map(|h| NodeExecutionHint {
      //     message: h,
      //     location: "outputPane",
      // }).collect());
    }

    Ok(())
  }
}

/// Split Out 数据拆分节点 V1
///
/// 将输入项中的数组或对象字段拆分为多个独立的输出项。
/// 支持多种拆分策略和字段配置选项。
///
/// # 技术特点
/// - 灵活的字段拆分策略
/// - 支持嵌套数据结构
/// - 多种字段包含模式
/// - 完善的错误处理
/// - 性能优化的数据处理
///
/// # 使用场景
/// - API 响应数据扁平化
/// - 批量数据准备
/// - 数据结构转换
/// - 工作流数据预处理
#[async_trait]
impl NodeExecutable for SplitOutV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 Split Out 节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Split Out 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::warn!("Split Out 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]));
    };

    // 解析配置参数
    let fields_to_split_str: String = node.get_parameter("fields_to_split")?;
    let fields_to_split = Self::parse_fields_to_split(&fields_to_split_str)?;

    let include_strategy_str: String = node.get_parameter("include_strategy")?;
    let include_strategy = match include_strategy_str.as_str() {
      "noOtherFields" => IncludeStrategy::NoOtherFields,
      "allOtherFields" => IncludeStrategy::AllOtherFields,
      "selectedOtherFields" => IncludeStrategy::SelectedOtherFields,
      _ => {
        return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
          "include_strategy",
          format!("Unknown strategy: {}", include_strategy_str),
        )));
      }
    };

    let fields_to_include_str: String = node.get_optional_parameter("fields_to_include").unwrap_or_default();
    let fields_to_include = prepare_fields_array(&fields_to_include_str, "Fields To Include");

    let disable_dot_notation: bool = node.get_optional_parameter("disable_dot_notation").unwrap_or(false);
    let include_binary: bool = node.get_optional_parameter("include_binary").unwrap_or(false);

    let config =
      SplitOutConfig { fields_to_split, include_strategy, fields_to_include, disable_dot_notation, include_binary };

    // 验证配置
    config.validate().map_err(|e| {
      NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
        "split_config",
        format!("Invalid configuration: {}", e),
      ))
    })?;

    log::info!(
      "Split Out 配置 - 拆分字段: {}, 策略: {}, 禁用点记号: {}, 包含二进制: {}",
      config.fields_to_split.len(),
      config.include_strategy,
      config.disable_dot_notation,
      config.include_binary
    );

    // 执行数据拆分
    self.execute_split_out(context, &input_items, &config).await
  }
}

impl SplitOutV1 {
  /// 解析要拆分的字段字符串
  fn parse_fields_to_split(fields_str: &str) -> Result<Vec<FieldToSplit>, NodeExecutionError> {
    if fields_str.trim().is_empty() {
      return Err(NodeExecutionError::ParameterValidation(ValidationError::required_field_missing("fields_to_split")));
    }

    let fields: Vec<&str> = fields_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if fields.is_empty() {
      return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
        "fields_to_split",
        "No fields specified".to_string(),
      )));
    }

    let mut result = Vec::new();
    for field in fields {
      result.push(FieldToSplit {
        field_to_split: field.to_string(),
        destination_field: None, // 默认使用源字段名
      });
    }

    Ok(result)
  }
}

impl TryFrom<NodeDefinition> for SplitOutV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .add_input(InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build())
      .add_output(OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build())
      .add_property(
        // 要拆分的字段
        NodeProperty::builder()
          .display_name("Fields to Split".to_string())
          .name("fields_to_split")
          .required(true)
          .description("The fields that should be split out".to_string())
          .placeholder("e.g. data.items, users")
          .kind(NodePropertyKind::String)
          .build(),
      )
      .add_property(
        // 字段包含策略
        NodeProperty::builder()
          .display_name("Include".to_string())
          .name("include_strategy")
          .required(true)
          .description("Choose which other fields the split out items should contain".to_string())
          .kind(NodePropertyKind::Options)
          .value(json!("noOtherFields"))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "No Other Fields",
              "noOtherFields",
              json!("noOtherFields"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "All Other Fields",
              "allOtherFields",
              json!("allOtherFields"),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Selected Other Fields",
              "selectedOtherFields",
              json!("selectedOtherFields"),
              NodePropertyKind::String,
            )),
          ])
          .build(),
      )
      .add_property(
        // 选择性包含字段
        NodeProperty::builder()
          .display_name("Fields to Include".to_string())
          .name("fields_to_include")
          .required(false)
          .description("The fields that split out items should contain".to_string())
          .placeholder("e.g. id, timestamp, status")
          .kind(NodePropertyKind::String)
          .build(),
      )
      .add_property(
        // 高级选项
        NodeProperty::builder()
          .display_name("Options".to_string())
          .name("options")
          .required(false)
          .description("Advanced options for data splitting".to_string())
          .placeholder("Add Option")
          .options(vec![
            Box::new(NodeProperty::new_option(
              "Disable Dot Notation",
              "disable_dot_notation",
              json!(false),
              NodePropertyKind::Boolean,
            )),
            Box::new(NodeProperty::new_option(
              "Include Binary",
              "include_binary",
              json!(false),
              NodePropertyKind::Boolean,
            )),
          ])
          .build(),
      );
    Ok(Self { definition: Arc::new(definition) })
  }
}
