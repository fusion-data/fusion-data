use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, FilterTypeOptions, InputPortConfig, NodeDefinition,
    NodeDefinitionBuilder, NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind,
    NodePropertyKindOptions, OutputPortConfig, RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{ConditionConfig, LogicCombination, utils::evaluate_single_condition};

#[derive(Debug)]
pub struct IfV1 {
  pub definition: Arc<NodeDefinition>,
}

impl IfV1 {
  /// 评估所有条件
  pub fn evaluate_conditions(
    &self,
    conditions: &[ConditionConfig],
    combination: &LogicCombination,
    input_data: &JsonValue,
  ) -> Result<bool, NodeExecutionError> {
    if conditions.is_empty() {
      return Ok(false);
    }

    let results: Result<Vec<bool>, NodeExecutionError> =
      conditions.iter().map(|condition| evaluate_single_condition(condition, input_data)).collect();

    let results = results?;

    let final_result = match combination {
      LogicCombination::And => results.iter().all(|&x| x),
      LogicCombination::Or => results.iter().any(|&x| x),
    };

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
    log::info!(
      "开始执行 If 条件判断节点 workflow_id:{}, node_name:{}, node_kind:{}",
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
      log::warn!("If 节点没有接收到输入数据");
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

    log::debug!("条件判断: {} 个条件，逻辑组合: {:?}", conditions.len(), logic_combination);

    // 最终结果
    let mut logic_value = true;

    for (index, input) in input_items.iter().enumerate() {
      let result = self.evaluate_conditions(&conditions, &logic_combination, input.json())?;
      logic_value = logic_value && result;
      log::debug!("输入数据项:{} 结果:{} 条件判断结果:{}", index, result, logic_value);
    }

    let res = if logic_value {
      vec![ExecutionDataItems::new_items(input_items), ExecutionDataItems::new_null()]
    } else {
      vec![ExecutionDataItems::new_null(), ExecutionDataItems::new_items(input_items)]
    };

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, res)]))
  }
}

impl TryFrom<NodeDefinitionBuilder> for IfV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .version(Version::new(1, 0, 0))
      .inputs([InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
      .outputs([
        OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("True").build(),
        OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("False").build(),
      ])
      .properties([
        NodeProperty::builder()
          .display_name("条件".to_string())
          .name("conditions")
          .required(true)
          .description("要评估的条件列表".to_string())
          .placeholder("添加条件...".to_string())
          .kind(NodePropertyKind::Filter)
          .kind_options(
            NodePropertyKindOptions::builder()
              .filter(
                FilterTypeOptions::builder().case_sensitive(json!("={{!$parameter.options.ignore_case}}")).build(),
              )
              .build(),
          )
          .build(),
        NodeProperty::builder()
          .display_name("Options")
          .name("options")
          .required(false)
          .placeholder("Add Option")
          .options(vec![Box::new(NodeProperty::new_option(
            "Ignore Case",
            "ignore_case",
            json!(true),
            NodePropertyKind::Boolean,
          ))])
          .build(),
        NodeProperty::builder()
          .display_name("逻辑组合".to_string())
          .name("combination")
          .kind(NodePropertyKind::Options)
          .required(false)
          .description("多个条件之间的逻辑关系".to_string())
          .value(json!(LogicCombination::And))
          .options(vec![
            Box::new(NodeProperty::new_option("AND", "and", json!(LogicCombination::And), NodePropertyKind::Boolean)),
            Box::new(NodeProperty::new_option("OR", "or", json!(LogicCombination::Or), NodePropertyKind::Boolean)),
          ])
          .placeholder("".to_string())
          .build(),
      ]);

    let definition = base.build()?;

    Ok(Self { definition: Arc::new(definition) })
  }
}
