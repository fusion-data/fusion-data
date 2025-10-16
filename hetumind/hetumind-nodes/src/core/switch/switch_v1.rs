use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, FilterTypeOptions, InputPortConfig, NodeDefinition,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, NodePropertyKindOptions,
    OutputPortConfig, RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  FallbackOutput, SwitchConfig, SwitchMode, SwitchOptions,
  utils::{evaluate_expression, evaluate_rules, handle_fallback_output},
};

/// Switch 条件路由节点 V1
///
/// 根据条件或表达式将输入数据路由到不同的输出端口。
/// 支持两种工作模式：Rules（规则）模式和 Expression（表达式）模式。
///
/// # Rules 模式
/// 基于条件集合的路由决策，支持多个规则和复杂条件组合。
///
/// # Expression 模式
/// 基于表达式计算的直接路由，根据表达式结果输出到指定端口。
///
/// # 输出端口
/// - 根据配置动态生成输出端口
/// - Rules 模式：基于规则数量和 fallback 配置
/// - Expression 模式：基于 numberOutputs 参数
#[derive(Debug)]
#[allow(dead_code)]
pub struct SwitchV1 {
  pub definition: Arc<NodeDefinition>,
}

impl SwitchV1 {
  /// 配置输出端口
  pub fn configure_outputs(&self, config: &SwitchConfig) -> Vec<OutputPortConfig> {
    match config.mode {
      SwitchMode::Rules => {
        let empty_rules = vec![];
        let rules = config.rules.as_ref().unwrap_or(&empty_rules);
        let mut outputs = Vec::new();

        for (index, rule) in rules.iter().enumerate() {
          let index_str = index.to_string();
          let display_name = rule.output_key.as_deref().unwrap_or(&index_str);
          outputs.push(OutputPortConfig::new(ConnectionKind::Main, display_name));
        }

        // 添加 fallback 输出端口
        if let Some(FallbackOutput::Extra) = config.options.fallback_output {
          outputs.push(OutputPortConfig::new(ConnectionKind::Main, "Fallback"));
        }

        outputs
      }
      SwitchMode::Expression => {
        let number_outputs = config.number_outputs.unwrap_or(1);
        (0..number_outputs).map(|i| OutputPortConfig::new(ConnectionKind::Main, &i.to_string())).collect()
      }
    }
  }

  /// 处理 Rules 模式
  #[allow(unused_variables)]
  async fn process_rules_mode(
    &self,
    _context: &NodeExecutionContext,
    config: &SwitchConfig,
    input_items: &[hetumind_core::workflow::ExecutionData],
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    let rules = config.rules.as_ref().unwrap();
    let total_outputs = rules.len();
    let mut output_data: Vec<Option<Vec<hetumind_core::workflow::ExecutionData>>> = vec![None; total_outputs];

    for input_item in input_items {
      let matched_outputs = evaluate_rules(rules, &config.options, input_item.json())?;

      if matched_outputs.is_empty() {
        // 处理 fallback
        handle_fallback_output(
          &config.options.fallback_output,
          total_outputs,
          &mut output_data,
          std::slice::from_ref(input_item),
        )?;
      } else {
        // 将数据添加到匹配的输出端口
        for output_index in matched_outputs {
          if output_index < total_outputs {
            if output_data[output_index].is_none() {
              output_data[output_index] = Some(vec![]);
            }
            if let Some(ref mut data) = output_data[output_index] {
              data.push(input_item.clone());
            }
          }
        }
      }
    }

    // 转换为 ExecutionDataMap
    let execution_outputs: Vec<ExecutionDataItems> = output_data
      .into_iter()
      .map(|data| match data {
        Some(d) => ExecutionDataItems::new_items(d),
        None => ExecutionDataItems::new_null(),
      })
      .collect();

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, execution_outputs)]))
  }

  /// 处理 Expression 模式
  #[allow(unused_variables)]
  async fn process_expression_mode(
    &self,
    _context: &NodeExecutionContext,
    config: &SwitchConfig,
    input_items: &[hetumind_core::workflow::ExecutionData],
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    let number_outputs = config.number_outputs.unwrap();
    let expression = config.output_expression.as_ref().unwrap();
    let mut output_data: Vec<Option<Vec<hetumind_core::workflow::ExecutionData>>> = vec![None; number_outputs];

    for input_item in input_items {
      let output_index = evaluate_expression(expression, number_outputs, input_item.json())?;

      if output_data[output_index].is_none() {
        output_data[output_index] = Some(vec![]);
      }
      if let Some(ref mut data) = output_data[output_index] {
        data.push(input_item.clone());
      }
    }

    // 转换为 ExecutionDataMap
    let execution_outputs: Vec<ExecutionDataItems> = output_data
      .into_iter()
      .map(|data| match data {
        Some(d) => ExecutionDataItems::new_items(d),
        None => ExecutionDataItems::new_null(),
      })
      .collect();

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, execution_outputs)]))
  }
}

#[async_trait]
impl NodeExecutable for SwitchV1 {
  #[allow(unused_variables)]
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Switch 条件路由节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      input_data
    } else {
      log::warn!("Switch 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_null()])]));
    };

    // 获取配置
    let mode: SwitchMode = node.get_parameter("mode")?;
    let mut config = SwitchConfig {
      mode,
      rules: None,
      number_outputs: None,
      output_expression: None,
      options: SwitchOptions::default(),
    };

    // 根据模式获取相应配置
    match config.mode {
      SwitchMode::Rules => {
        // 获取规则配置
        let rules: Vec<serde_json::Value> = node.get_parameter("rules")?;
        let mut switch_rules = Vec::new();

        for (index, rule_value) in rules.into_iter().enumerate() {
          if let serde_json::Value::Object(_rule_obj) = rule_value {
            let conditions: Vec<crate::core::if_node::ConditionConfig> =
              node.get_parameter(&format!("rules.values[{}].conditions", index))?;
            let output_key: Option<String> = node.get_optional_parameter(&format!("rules.values[{}].outputKey", index));

            switch_rules.push(super::SwitchRule { output_key, conditions, output_index: Some(index) });
          }
        }

        config.rules = Some(switch_rules);

        // 获取选项配置
        let all_matching_outputs: Option<bool> = node.get_optional_parameter("options.allMatchingOutputs");
        let ignore_case: Option<bool> = node.get_optional_parameter("options.ignoreCase");
        let loose_type_validation: Option<bool> = node.get_optional_parameter("options.looseTypeValidation");
        let fallback_output_value: Option<serde_json::Value> = node.get_optional_parameter("options.fallbackOutput");

        let fallback_output = match fallback_output_value {
          Some(value) => {
            if let Some(s) = value.as_str() {
              match s {
                "none" => Some(FallbackOutput::None),
                "extra" => Some(FallbackOutput::Extra),
                _ => None, // 对于数字端口，需要更复杂的处理
              }
            } else {
              value.as_u64().map(|port| FallbackOutput::Port(port as usize))
            }
          }
          None => Some(FallbackOutput::None),
        };

        config.options = SwitchOptions { all_matching_outputs, ignore_case, loose_type_validation, fallback_output };
      }
      SwitchMode::Expression => {
        // 获取表达式配置
        let number_outputs: usize = node.get_parameter("numberOutputs")?;
        let output_expression: JsonValue = node.get_parameter("outputExpression")?;

        config.number_outputs = Some(number_outputs);
        config.output_expression = Some(output_expression);
      }
    }

    // 验证配置
    if let Err(e) = config.validate() {
      return Err(NodeExecutionError::DataProcessingError { message: format!("Invalid switch configuration: {}", e) });
    }

    log::debug!("Switch 配置: 模式={:?}, 输入数据量={}", config.mode, input_items.len());

    // 根据模式处理数据
    let result = match config.mode {
      SwitchMode::Rules => self.process_rules_mode(context, &config, &input_items).await,
      SwitchMode::Expression => self.process_expression_mode(context, &config, &input_items).await,
    };

    match &result {
      Ok(_data_map) => {
        log::info!("Switch 节点执行完成: 输入 {} 项", input_items.len());
      }
      Err(e) => {
        log::error!("Switch 节点执行失败: {}", e);
      }
    }

    result
  }
}

impl TryFrom<NodeDefinition> for SwitchV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Output 0"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Output 1"))
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("模式")
          .with_name("mode")
          .with_required(true)
          .with_description("Switch 节点的工作模式")
          .with_value(json!(SwitchMode::Rules))
          .with_options(vec![
            Box::new(NodeProperty::new_option("Rules", "rules", json!(SwitchMode::Rules), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "Expression",
              "expression",
              json!(SwitchMode::Expression),
              NodePropertyKind::String,
            )),
          ]),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Filter)
          .with_display_name("规则")
          .with_name("rules")
          .with_required(false)
          .with_description("路由规则集合（Rules 模式）")
          .with_placeholder("添加规则...")
          .with_kind_options(hetumind_core::workflow::NodePropertyKindOptions {
            filter: Some(
              hetumind_core::workflow::FilterTypeOptions::new()
                .with_case_sensitive(json!("={{!$parameter.options.ignoreCase}}")),
            ),
            button_config: None,
            container_class: None,
            always_open_edit_window: None,
            code_autocomplete: None,
            editor: None,
            editor_is_read_only: None,
            sql_dialect: None,
            load_options_depends_on: None,
            load_options_method: None,
            load_options: None,
            max_value: None,
            min_value: None,
            multiple_values: None,
            multiple_value_button_text: None,
            number_precision: None,
            password: None,
            rows: None,
            show_alpha: None,
            sortable: None,
            expirable: None,
            resource_mapper: None,
            assignment: None,
            min_required_fields: None,
            max_allowed_fields: None,
            callout_action: None,
            additional_properties: serde_json::Map::new(),
          }),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("输出数量")
          .with_name("numberOutputs")
          .with_required(false)
          .with_description("输出端口数量（Expression 模式）")
          .with_value(json!(2)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("输出表达式")
          .with_name("outputExpression")
          .with_required(false)
          .with_description("用于计算输出索引的表达式（Expression 模式）")
          .with_placeholder("{{ $json.index }}"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::FixedCollection)
          .with_display_name("选项")
          .with_name("options")
          .with_required(false)
          .with_placeholder("添加选项")
          .with_options(vec![
            Box::new(NodeProperty::new_option(
              "All Matching Outputs",
              "allMatchingOutputs",
              json!(false),
              NodePropertyKind::Boolean,
            )),
            Box::new(NodeProperty::new_option("Ignore Case", "ignoreCase", json!(false), NodePropertyKind::Boolean)),
            Box::new(NodeProperty::new_option(
              "Loose Type Validation",
              "looseTypeValidation",
              json!(false),
              NodePropertyKind::Boolean,
            )),
            Box::new(NodeProperty::new_option(
              "Fallback Output",
              "fallbackOutput",
              json!("none"),
              NodePropertyKind::Options,
            )),
          ]),
      );
    Ok(Self { definition: Arc::new(definition) })
  }
}
