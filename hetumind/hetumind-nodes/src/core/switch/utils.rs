//! Switch 节点工具函数

use log::debug;

use hetumind_core::types::JsonValue;
use hetumind_core::workflow::NodeExecutionError;

use crate::core::if_node::{ConditionConfig, utils::evaluate_single_condition};

/// 评估规则集合
pub fn evaluate_rules(
  rules: &[crate::core::switch::SwitchRule],
  options: &crate::core::switch::SwitchOptions,
  input_data: &JsonValue,
) -> Result<Vec<usize>, NodeExecutionError> {
  let mut matched_outputs = Vec::new();
  let all_matching = options.all_matching_outputs.unwrap_or(false);

  for (rule_index, rule) in rules.iter().enumerate() {
    // 评估规则的所有条件
    let rule_matches = evaluate_rule_conditions(&rule.conditions, input_data)?;

    if rule_matches {
      let output_index = rule.output_index.unwrap_or(rule_index);
      matched_outputs.push(output_index);

      debug!("规则 {} 匹配，输出到端口 {}", rule_index, output_index);

      // 如果不是所有匹配模式，找到第一个匹配就返回
      if !all_matching {
        break;
      }
    }
  }

  Ok(matched_outputs)
}

/// 评估单个规则的所有条件（使用 AND 逻辑）
fn evaluate_rule_conditions(
  conditions: &[ConditionConfig],
  input_data: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  if conditions.is_empty() {
    return Ok(false);
  }

  for condition in conditions {
    if !evaluate_single_condition(condition, input_data)? {
      return Ok(false);
    }
  }

  Ok(true)
}

/// 计算表达式并返回输出索引
pub fn evaluate_expression(
  expression: &JsonValue,
  number_outputs: usize,
  input_data: &JsonValue,
) -> Result<usize, NodeExecutionError> {
  // 简单的表达式评估实现
  // 在实际应用中，这里应该使用更完整的表达式引擎
  let output_index = match expression {
    JsonValue::String(expr) => {
      // 简单的 JSON Path 表达式支持
      if let Some(path) = expr.strip_prefix("{{")
        && let Some(path) = path.strip_suffix("}}")
      {
        let path = path.trim().strip_prefix("$json.").unwrap_or(path.trim());
        let value = get_nested_value(input_data, path).unwrap_or(JsonValue::Null);

        // 尝试将值转换为数字
        match value {
          JsonValue::Number(n) => n.as_u64().unwrap_or(0) as usize,
          JsonValue::String(s) => s.parse().unwrap_or(0),
          _ => 0,
        }
      } else {
        // 直接尝试解析为数字
        expr.parse().unwrap_or(0)
      }
    }
    JsonValue::Number(n) => n.as_u64().unwrap_or(0) as usize,
    _ => 0,
  };

  // 验证输出索引范围
  if output_index >= number_outputs {
    return Err(NodeExecutionError::DataProcessingError {
      message: format!("Output index {} is out of range. Valid range is 0 to {}", output_index, number_outputs - 1),
    });
  }

  Ok(output_index)
}

/// 获取嵌套值
fn get_nested_value(data: &JsonValue, path: &str) -> Option<JsonValue> {
  let parts: Vec<&str> = path.split('.').collect();
  let mut current = data;

  for part in parts {
    match current {
      JsonValue::Object(obj) => {
        current = obj.get(part)?;
      }
      _ => return None,
    }
  }

  Some(current.clone())
}

/// 处理 fallback 输出
pub fn handle_fallback_output(
  fallback: &Option<crate::core::switch::FallbackOutput>,
  total_outputs: usize,
  output_data: &mut Vec<Option<Vec<hetumind_core::workflow::ExecutionData>>>,
  input_data: &[hetumind_core::workflow::ExecutionData],
) -> Result<(), NodeExecutionError> {
  match fallback {
    Some(crate::core::switch::FallbackOutput::None) => {
      // 不处理，忽略数据
      Ok(())
    }
    Some(crate::core::switch::FallbackOutput::Extra) => {
      // 输出到额外的备用端口
      if output_data.len() <= total_outputs {
        output_data.resize(total_outputs + 1, None);
      }
      output_data[total_outputs] = Some(input_data.to_vec());
      Ok(())
    }
    Some(crate::core::switch::FallbackOutput::Port(port_index)) => {
      // 输出到指定端口
      if *port_index >= total_outputs {
        return Err(NodeExecutionError::DataProcessingError {
          message: format!(
            "Fallback port index {} is out of range. Valid range is 0 to {}",
            port_index,
            total_outputs - 1
          ),
        });
      }
      output_data[*port_index] = Some(input_data.to_vec());
      Ok(())
    }
    None => {
      // 默认忽略数据
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::types::DataType;
  use hetumind_core::workflow::ExecutionData;
  use serde_json::json;

  use super::*;
  use crate::core::if_node::{ComparisonOperation, ConditionConfig};
  use crate::core::switch::{FallbackOutput, SwitchOptions, SwitchRule};

  #[test]
  fn test_evaluate_expression() {
    let input_data = json!({"index": 2, "name": "test"});

    // 测试数字表达式
    let result = evaluate_expression(&json!(1), 3, &input_data).unwrap();
    assert_eq!(result, 1);

    // 测试 JSON Path 表达式
    let result = evaluate_expression(&json!("{{ $json.index }}"), 3, &input_data).unwrap();
    assert_eq!(result, 2);

    // 测试超出范围
    let result = evaluate_expression(&json!(5), 3, &input_data);
    assert!(result.is_err());
  }

  #[test]
  fn test_evaluate_rules() {
    let input_data = json!({"type": "A", "value": 10});

    let rules = vec![
      SwitchRule {
        output_key: Some("output1".to_string()),
        conditions: vec![ConditionConfig {
          left: json!("A"), // 直接使用值而不是表达式
          op: ComparisonOperation::Eq,
          right: Some(json!("A")),
          data_type: DataType::String,
        }],
        output_index: Some(0),
      },
      SwitchRule {
        output_key: Some("output2".to_string()),
        conditions: vec![ConditionConfig {
          left: json!(10), // 直接使用值而不是表达式
          op: ComparisonOperation::Gt,
          right: Some(json!(5)),
          data_type: DataType::Number,
        }],
        output_index: Some(1),
      },
    ];

    let options = SwitchOptions {
      all_matching_outputs: Some(false),
      ignore_case: Some(false),
      loose_type_validation: Some(false),
      fallback_output: None,
    };

    let result = evaluate_rules(&rules, &options, &input_data).unwrap();
    assert_eq!(result, vec![0]); // 只匹配第一个规则（非 all_matching 模式）

    let options_all_matching = SwitchOptions {
      all_matching_outputs: Some(true),
      ignore_case: Some(false),
      loose_type_validation: Some(false),
      fallback_output: None,
    };

    let result = evaluate_rules(&rules, &options_all_matching, &input_data).unwrap();
    assert_eq!(result, vec![0, 1]); // 两个规则都匹配
  }

  #[test]
  fn test_handle_fallback_output() {
    let input_data = vec![ExecutionData::new_json(json!({"test": "data"}), None)];
    let mut output_data = vec![None, None];

    // 测试 None fallback
    let result = handle_fallback_output(&None, 2, &mut output_data, &input_data);
    assert!(result.is_ok());
    assert!(output_data[0].is_none());
    assert!(output_data[1].is_none());

    // 测试 Extra fallback
    let result = handle_fallback_output(&Some(FallbackOutput::Extra), 2, &mut output_data, &input_data);
    assert!(result.is_ok());
    assert!(output_data[2].is_some());
    assert_eq!(output_data[2].as_ref().unwrap().len(), 1);

    // 测试 Port fallback
    output_data = vec![None, None];
    let result = handle_fallback_output(&Some(FallbackOutput::Port(1)), 2, &mut output_data, &input_data);
    assert!(result.is_ok());
    assert!(output_data[1].is_some());
    assert_eq!(output_data[1].as_ref().unwrap().len(), 1);

    // 测试超出范围的端口
    let result = handle_fallback_output(&Some(FallbackOutput::Port(5)), 2, &mut output_data, &input_data);
    assert!(result.is_err());
  }

  #[test]
  fn test_get_nested_value() {
    let data = json!({
      "user": {
        "name": "Alice",
        "profile": {
          "age": 25
        }
      }
    });

    let result = get_nested_value(&data, "user.name");
    assert_eq!(result, Some(json!("Alice")));

    let result = get_nested_value(&data, "user.profile.age");
    assert_eq!(result, Some(json!(25)));

    let result = get_nested_value(&data, "user.nonexistent");
    assert_eq!(result, None);
  }
}
