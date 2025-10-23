use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition, NodeExecutable,
    NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig, RegistrationError,
    make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  AdvancedConditionGroup, ConditionConfig, ErrorHandlingStrategy, IfNodeOptions, LogicCombination,
  utils::evaluate_single_condition_with_options,
};

#[derive(Debug)]
pub struct IfV1 {
  pub definition: Arc<NodeDefinition>,
}

impl IfV1 {
  /// 评估所有条件（带配置选项）
  pub fn evaluate_conditions_with_options(
    &self,
    conditions: &[ConditionConfig],
    combination: &LogicCombination,
    input_data: &JsonValue,
    options: &IfNodeOptions,
  ) -> Result<bool, NodeExecutionError> {
    if conditions.is_empty() {
      return Ok(false);
    }

    let results: Result<Vec<bool>, NodeExecutionError> = conditions
      .iter()
      .map(|condition| evaluate_single_condition_with_options(condition, input_data, options))
      .collect();

    let results = results?;

    let final_result = match combination {
      LogicCombination::And => results.iter().all(|&x| x),
      LogicCombination::Or => results.iter().any(|&x| x),
    };

    Ok(final_result)
  }

  /// 评估所有条件（保持向后兼容）
  #[allow(dead_code)]
  pub fn evaluate_conditions(
    &self,
    conditions: &[ConditionConfig],
    combination: &LogicCombination,
    input_data: &JsonValue,
  ) -> Result<bool, NodeExecutionError> {
    let options = IfNodeOptions::default();
    self.evaluate_conditions_with_options(conditions, combination, input_data, &options)
  }

  /// 评估高级条件组合
  pub fn evaluate_advanced_conditions(
    &self,
    advanced_group: &AdvancedConditionGroup,
    input_data: &JsonValue,
    options: &IfNodeOptions,
  ) -> Result<bool, NodeExecutionError> {
    advanced_group.evaluate(input_data, options)
  }

  /// 混合条件评估（支持简单条件和高级条件组合）
  pub fn evaluate_mixed_conditions(
    &self,
    simple_conditions: &[ConditionConfig],
    simple_combination: &LogicCombination,
    advanced_groups: &[AdvancedConditionGroup],
    input_data: &JsonValue,
    options: &IfNodeOptions,
  ) -> Result<bool, NodeExecutionError> {
    let mut all_results = Vec::new();

    // 评估简单条件
    if !simple_conditions.is_empty() {
      let simple_result =
        self.evaluate_conditions_with_options(simple_conditions, simple_combination, input_data, options)?;
      all_results.push(simple_result);

      if options.debug_mode {
        log::debug!("简单条件评估结果: {}", simple_result);
      }
    }

    // 评估高级条件组合
    for (index, advanced_group) in advanced_groups.iter().enumerate() {
      let advanced_result = self.evaluate_advanced_conditions(advanced_group, input_data, options)?;
      all_results.push(advanced_result);

      if options.debug_mode {
        if let Some(name) = &advanced_group.name {
          log::debug!("高级条件组 '{}' (#{}) 评估结果: {}", name, index, advanced_result);
        } else {
          log::debug!("高级条件组 #{} 评估结果: {}", index, advanced_result);
        }
      }
    }

    // 默认使用 AND 逻辑组合所有结果
    let final_result = all_results.iter().all(|&x| x);

    if options.debug_mode {
      log::debug!("混合条件最终评估结果: {} ({} 个结果)", final_result, all_results.len());
    }

    Ok(final_result)
  }
}

/// If 条件判断节点
///
/// 用于根据条件判断将工作流分为 true 和 false 两个分支。
/// 支持多种数据类型的比较操作，以及 AND/OR 逻辑组合。
///
/// # 输出分支
/// - `true`: 条件满足时的输出分支
/// - `false`: 条件不满足时的输出分支
///
/// # 支持的数据类型
/// - String: 字符串比较
/// - Number: 数值比较
/// - Boolean: 布尔值比较
/// - DateTime: 日期时间比较
///
/// # 支持的比较操作
/// - equal: 等于
/// - notEqual: 不等于
/// - contains: 包含
/// - notContains: 不包含
/// - startsWith: 以...开始
/// - endsWith: 以...结束
/// - regex: 正则表达式匹配
/// - isEmpty: 为空
/// - isNotEmpty: 不为空
/// - greaterThan: 大于
/// - lessThan: 小于
/// - greaterThanOrEqual: 大于等于
/// - lessThanOrEqual: 小于等于
#[async_trait]
impl NodeExecutable for IfV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    println!(
      "[DEBUG] 开始执行 If 条件判断节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("If 节点接收到 {} 个输入项", input_data.len());
      for (i, item) in input_data.iter().enumerate() {
        log::info!("输入项 {}: {}", i, serde_json::to_string(item.json()).unwrap_or_default());
      }
      input_data
    } else {
      log::error!("If 节点没有接收到输入数据");
      // 如果没有输入数据，默认走 false 分支
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_null(), ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取条件配置
    let conditions: Vec<ConditionConfig> = node.get_parameter("conditions")?;
    let logic_combination: LogicCombination =
      node.get_optional_parameter("combination").unwrap_or(LogicCombination::And);

    // 尝试获取高级条件组合（可选）
    let advanced_groups: Vec<AdvancedConditionGroup> =
      node.get_optional_parameter("advanced_condition_groups").unwrap_or_default();

    // 获取配置选项
    let options: IfNodeOptions = node.get_optional_parameter("options").unwrap_or_default();

    log::info!(
      "条件判断 - 简单条件: {} 个, 高级条件组: {} 个, 逻辑组合: {:?}, 配置: {:?}",
      conditions.len(),
      advanced_groups.len(),
      logic_combination,
      options
    );

    // 打印条件配置详细信息
    if !conditions.is_empty() {
      log::info!("条件配置详情: {:?}", serde_json::to_string(&conditions).unwrap_or_default());
    }

    // 修复：对每个输入项独立处理，而不是共享同一个结果
    let mut true_items = Vec::new();
    let mut false_items = Vec::new();
    let mut skipped_items = Vec::new();

    for (index, input) in input_items.iter().enumerate() {
      log::info!("开始处理输入项 {}: {}", index, serde_json::to_string(input.json()).unwrap_or_default());

      let evaluation_result = if !advanced_groups.is_empty() && !conditions.is_empty() {
        // 混合条件评估
        log::info!("使用混合条件评估模式");
        self.evaluate_mixed_conditions(&conditions, &logic_combination, &advanced_groups, input.json(), &options)
      } else if !advanced_groups.is_empty() {
        // 仅高级条件评估
        log::info!("使用高级条件评估模式");
        let mut advanced_results = Vec::new();
        for advanced_group in &advanced_groups {
          let result = self.evaluate_advanced_conditions(advanced_group, input.json(), &options)?;
          advanced_results.push(result);
        }
        // 高级条件组之间使用 AND 逻辑
        Ok(advanced_results.iter().all(|&x| x))
      } else {
        // 传统简单条件评估
        log::info!("使用简单条件评估模式");
        self.evaluate_conditions_with_options(&conditions, &logic_combination, input.json(), &options)
      };

      match evaluation_result {
        Ok(result) => {
          if result {
            true_items.push(input.clone());
            if options.debug_mode {
              log::debug!("输入数据项:{} 结果:true -> 分配到 true 分支", index);
            }
          } else {
            false_items.push(input.clone());
            if options.debug_mode {
              log::debug!("输入数据项:{} 结果:false -> 分配到 false 分支", index);
            }
          }
        }
        Err(e) => match options.error_handling_strategy {
          ErrorHandlingStrategy::SkipItem => {
            skipped_items.push(input.clone());
            log::warn!("输入数据项:{} 条件评估失败: {} -> 跳过该项", index, e);
          }
          ErrorHandlingStrategy::StopExecution => {
            log::error!("输入数据项:{} 条件评估失败: {} -> 停止执行", index, e);
            return Err(e);
          }
          ErrorHandlingStrategy::GoToDefaultBranch => {
            false_items.push(input.clone());
            log::error!("输入数据项:{} 条件评估失败: {} -> 分配到 false 分支", index, e);
          }
          ErrorHandlingStrategy::LogAndContinue => {
            false_items.push(input.clone());
            log::warn!("输入数据项:{} 条件评估失败: {} -> 记录错误但继续", index, e);
          }
        },
      }
    }

    // 记录执行统计
    log::info!(
      "条件评估完成 - true 分支: {} 项, false 分支: {} 项, 跳过: {} 项",
      true_items.len(),
      false_items.len(),
      skipped_items.len()
    );

    // 处理跳过的项目
    if !skipped_items.is_empty() {
      match options.error_handling_strategy {
        ErrorHandlingStrategy::SkipItem => {
          // 跳过的项目不进入任何分支
          log::info!("跳过的项目将被丢弃，不进入任何输出分支");
        }
        _ => {
          // 其他策略已经在上面的循环中处理
        }
      }
    }

    log::info!("条件评估完成 - true 分支: {} 项, false 分支: {} 项", true_items.len(), false_items.len());

    // 根据实际评估结果分发数据
    let res = vec![
      ExecutionDataItems::new_items(true_items),  // true 分支
      ExecutionDataItems::new_items(false_items), // false 分支
    ];

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, res)]))
  }
}

impl TryFrom<NodeDefinition> for IfV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "True"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "False"))
      .add_property(
        NodeProperty::new(NodePropertyKind::Filter)
          .with_display_name("条件")
          .with_name("conditions")
          .with_required(true)
          .with_description("要评估的条件列表")
          .with_placeholder("添加条件...")
          .with_kind_options(hetumind_core::workflow::NodePropertyKindOptions {
            filter: Some(
              hetumind_core::workflow::FilterTypeOptions::new()
                .with_case_sensitive(json!("={{!$parameter.options.ignore_case}}")),
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
        NodeProperty::new(NodePropertyKind::Collection)
          .with_display_name("Options")
          .with_name("options")
          .with_required(false)
          .with_placeholder("Add Option")
          .add_option(Box::new(NodeProperty::new_option(
            "Ignore Case",
            "ignore_case",
            json!(true),
            NodePropertyKind::Boolean,
          ))),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("逻辑组合")
          .with_name("combination")
          .with_required(false)
          .with_description("多个条件之间的逻辑关系")
          .with_value(json!(LogicCombination::And))
          .add_option(Box::new(NodeProperty::new_option(
            "AND",
            "and",
            json!(LogicCombination::And),
            NodePropertyKind::Boolean,
          )))
          .add_option(Box::new(NodeProperty::new_option(
            "OR",
            "or",
            json!(LogicCombination::Or),
            NodePropertyKind::Boolean,
          )))
          .with_placeholder(""),
      )
      .add_property(
        // 高级配置选项
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("高级选项")
          .with_name("advanced_options")
          .with_required(false)
          .with_description("IfNode 高级配置选项"),
      )
      .add_property(
        // 高级条件组合（V2 功能）
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("高级条件组合")
          .with_name("advanced_condition_groups")
          .with_required(false)
          .with_description("高级条件组合配置，支持复杂的条件逻辑组合")
          .with_placeholder("配置高级条件组..."),
      )
      .add_property(
        // 条件描述和配置
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("条件配置")
          .with_name("condition_config")
          .with_required(false)
          .with_description("条件的高级配置选项"),
      );

    Ok(Self { definition: Arc::new(definition) })
  }
}
