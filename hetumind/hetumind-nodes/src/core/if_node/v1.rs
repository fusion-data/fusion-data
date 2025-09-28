use std::sync::Arc;

use hetumind_core::{
  types::JsonValue,
  workflow::{
    ConnectionKind, FilterTypeOptions, InputPortConfig, NodeDefinition, NodeExecutionError, NodeProperties,
    NodePropertyKind, NodePropertyKindOptions, OutputPortConfig,
  },
};
use serde_json::json;

use super::{ConditionConfig, LogicCombination, utils::evaluate_single_condition};

#[derive(Debug)]
pub struct IfNodeV1 {
  pub definition: Arc<NodeDefinition>,
}

impl IfNodeV1 {
  pub fn new(mut definition: NodeDefinition) -> Self {
    definition.inputs = vec![InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()];
    definition.outputs = vec![
      OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("True").build(),
      OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("False").build(),
    ];
    definition.properties = vec![
      NodeProperties::builder()
        .display_name("条件".to_string())
        .name("conditions")
        .required(true)
        .description("要评估的条件列表".to_string())
        .placeholder("添加条件...".to_string())
        .kind(NodePropertyKind::Filter)
        .kind_options(
          NodePropertyKindOptions::builder()
            .filter(FilterTypeOptions::builder().case_sensitive(json!("={{!$parameter.options.ignore_case}}")).build())
            .build(),
        )
        .build(),
      NodeProperties::builder()
        .display_name("Options")
        .name("options")
        .required(false)
        .placeholder("Add Option")
        .options(vec![Box::new(NodeProperties::new_option(
          "Ignore Case",
          "ignore_case",
          json!(true),
          NodePropertyKind::Boolean,
        ))])
        .build(),
      NodeProperties::builder()
        .display_name("逻辑组合".to_string())
        .name("combination")
        .kind(NodePropertyKind::Options)
        .required(false)
        .description("多个条件之间的逻辑关系".to_string())
        .value(json!(LogicCombination::And))
        .options(vec![
          Box::new(NodeProperties::new_option("AND", "and", json!(LogicCombination::And), NodePropertyKind::Boolean)),
          Box::new(NodeProperties::new_option("OR", "or", json!(LogicCombination::Or), NodePropertyKind::Boolean)),
        ])
        .placeholder("".to_string())
        .build(),
    ];
    Self { definition: Arc::new(definition) }
  }

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
