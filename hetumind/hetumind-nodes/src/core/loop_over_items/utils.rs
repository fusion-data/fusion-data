//! Loop Over Items 节点工具函数

use serde_json::{Value, json};

use super::{LoopConfig, LoopMode};
use hetumind_core::workflow::{DataSource, ExecutionData, NodeExecutionError, NodeName};

/// 应用循环处理到数据上
pub fn process_loop(
  input_items: &[ExecutionData],
  config: &LoopConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  match config.mode {
    LoopMode::Items => process_items(input_items, config, current_node_name),
    LoopMode::Times => process_times(input_items, config, current_node_name),
    LoopMode::While => process_while(input_items, config, current_node_name),
    LoopMode::Batch => process_batch(input_items, config, current_node_name),
  }
}

/// Items 模式：对每个数据项执行一次循环
fn process_items(
  input_items: &[ExecutionData],
  config: &LoopConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let max_iterations = config.max_iterations.unwrap_or(input_items.len() as u32);
  let actual_iterations = std::cmp::min(input_items.len(), max_iterations as usize);

  let mut result = Vec::with_capacity(actual_iterations);

  for (index, data) in input_items.iter().take(actual_iterations).enumerate() {
    let mut loop_data = data.json().clone();

    // 如果需要包含索引，添加索引信息
    if config.include_index {
      if let Some(obj) = loop_data.as_object_mut() {
        obj.insert("index".to_string(), json!(index));
      } else {
        loop_data = json!({
          "index": index,
          "value": loop_data
        });
      }
    }

    result.push(ExecutionData::new_json(
      loop_data,
      Some(DataSource {
        node_name: current_node_name.clone(),
        output_port: hetumind_core::workflow::ConnectionKind::Main,
        output_index: index,
      }),
    ));
  }

  Ok(result)
}

/// Times 模式：固定次数循环
fn process_times(
  input_items: &[ExecutionData],
  config: &LoopConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let iterations = config.iterations.ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: "Iterations parameter is required for Times mode".to_string(),
  })?;

  let max_iterations = config.max_iterations.unwrap_or(iterations);
  let actual_iterations = std::cmp::min(iterations, max_iterations);

  let mut result = Vec::with_capacity(actual_iterations as usize);

  for index in 0..actual_iterations {
    let input_index = (index as usize) % input_items.len();
    let mut loop_data = input_items[input_index].json().clone();

    // 添加循环信息
    if let Some(obj) = loop_data.as_object_mut() {
      obj.insert("iteration".to_string(), json!(index));
    } else {
      loop_data = json!({
        "iteration": index,
        "value": loop_data
      });
    }

    if config.include_index {
      if let Some(obj) = loop_data.as_object_mut() {
        obj.insert("item_index".to_string(), json!(input_index));
      } else {
        loop_data = json!({
          "iteration": index,
          "item_index": input_index,
          "value": loop_data
        });
      }
    }

    result.push(ExecutionData::new_json(
      loop_data,
      Some(DataSource {
        node_name: current_node_name.clone(),
        output_port: hetumind_core::workflow::ConnectionKind::Main,
        output_index: index as usize,
      }),
    ));
  }

  Ok(result)
}

/// While 模式：条件循环
fn process_while(
  input_items: &[ExecutionData],
  config: &LoopConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let condition = config.condition.as_ref().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: "Condition parameter is required for While mode".to_string(),
  })?;

  let max_iterations = config.max_iterations.unwrap_or(1000);
  let mut result = Vec::new();
  let mut iteration = 0;

  while iteration < max_iterations {
    if input_items.is_empty() {
      break;
    }

    let current_data = &input_items[iteration as usize % input_items.len()];
    let mut loop_data = current_data.json().clone();

    // 添加循环信息
    if let Some(obj) = loop_data.as_object_mut() {
      obj.insert("iteration".to_string(), json!(iteration));
    } else {
      loop_data = json!({
        "iteration": iteration,
        "value": loop_data
      });
    }

    if config.include_index {
      let input_index = iteration as usize % input_items.len();
      if let Some(obj) = loop_data.as_object_mut() {
        obj.insert("item_index".to_string(), json!(input_index));
      } else {
        loop_data = json!({
          "iteration": iteration,
          "item_index": input_index,
          "value": loop_data
        });
      }
    }

    result.push(ExecutionData::new_json(
      loop_data.clone(),
      Some(DataSource {
        node_name: current_node_name.clone(),
        output_port: hetumind_core::workflow::ConnectionKind::Main,
        output_index: iteration as usize,
      }),
    ));

    // 简单的条件评估（在实际实现中可能需要更复杂的表达式解析）
    if !evaluate_condition(&loop_data, condition) {
      break;
    }

    iteration += 1;
  }

  Ok(result)
}

/// Batch 模式：批量处理
fn process_batch(
  input_items: &[ExecutionData],
  config: &LoopConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let batch_size = config.batch_size.ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: "Batch size parameter is required for Batch mode".to_string(),
  })?;

  let max_iterations = config.max_iterations.unwrap_or(1000);
  let batch_count = input_items.len().div_ceil(batch_size);
  let max_iterations = std::cmp::min(max_iterations, batch_count as u32);

  let mut result = Vec::new();

  for batch_index in 0..max_iterations {
    let start_index = batch_index as usize * batch_size;
    let end_index = std::cmp::min(start_index + batch_size, input_items.len());

    if start_index >= input_items.len() {
      break;
    }

    let batch_data: Vec<&ExecutionData> = input_items[start_index..end_index].iter().collect();
    let mut batch_json = json!({
      "batch_index": batch_index,
      "items": Vec::<Value>::new(),
    });

    // 添加批次中的所有数据项
    for (item_index, data) in batch_data.iter().enumerate() {
      let mut item_data = data.json().clone();

      if config.include_index {
        if let Some(obj) = item_data.as_object_mut() {
          obj.insert("index".to_string(), json!(start_index + item_index));
        } else {
          item_data = json!({
            "index": start_index + item_index,
            "value": item_data
          });
        }
      }

      if let Some(items_array) = batch_json["items"].as_array_mut() {
        items_array.push(item_data);
      }
    }

    result.push(ExecutionData::new_json(
      batch_json,
      Some(DataSource {
        node_name: current_node_name.clone(),
        output_port: hetumind_core::workflow::ConnectionKind::Main,
        output_index: batch_index as usize,
      }),
    ));
  }

  Ok(result)
}

/// 简单的条件评估函数
fn evaluate_condition(data: &Value, condition: &str) -> bool {
  // 这是一个简化的条件评估实现
  // 在实际使用中，可能需要更复杂的表达式解析器

  // 检查是否为布尔值
  if let Some(bool_value) = data.get(condition)
    && let Some(b) = bool_value.as_bool()
  {
    return b;
  }

  // 检查字符串值
  if let Some(str_value) = data.get(condition).and_then(|v| v.as_str()) {
    return !str_value.is_empty();
  }

  // 检查数值是否大于0
  if let Some(num_value) = data.get(condition).and_then(|v| v.as_f64()) {
    return num_value > 0.0;
  }

  // 检查是否存在字段（非空、非零值）
  data.get(condition).is_some()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_process_items() {
    let input_items = vec![
      ExecutionData::new_json(json!({"name": "Alice", "age": 25}), None),
      ExecutionData::new_json(json!({"name": "Bob", "age": 30}), None),
      ExecutionData::new_json(json!({"name": "Charlie", "age": 35}), None),
    ];

    let config = LoopConfig {
      mode: LoopMode::Items,
      iterations: None,
      batch_size: None,
      condition: None,
      max_iterations: Some(2),
      include_index: false,
      parallel: false,
    };

    let result = process_items(&input_items, &config, &"test".into()).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].json()["name"], "Alice");
    assert_eq!(result[1].json()["name"], "Bob");
  }

  #[test]
  fn test_process_times() {
    let input_items = vec![
      ExecutionData::new_json(json!({"name": "Alice"}), None),
      ExecutionData::new_json(json!({"name": "Bob"}), None),
    ];

    let config = LoopConfig {
      mode: LoopMode::Times,
      iterations: Some(3),
      batch_size: None,
      condition: None,
      max_iterations: Some(5),
      include_index: true,
      parallel: false,
    };

    let result = process_times(&input_items, &config, &"test".into()).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].json()["iteration"], 0);
    assert_eq!(result[0].json()["item_index"], 0);
    assert_eq!(result[1].json()["iteration"], 1);
    assert_eq!(result[1].json()["item_index"], 1);
    assert_eq!(result[2].json()["iteration"], 2);
    assert_eq!(result[2].json()["item_index"], 0); // 循环使用第一个元素
  }

  #[test]
  fn test_process_batch() {
    let input_items = vec![
      ExecutionData::new_json(json!({"id": 1, "name": "Alice"}), None),
      ExecutionData::new_json(json!({"id": 2, "name": "Bob"}), None),
      ExecutionData::new_json(json!({"id": 3, "name": "Charlie"}), None),
      ExecutionData::new_json(json!({"id": 4, "name": "David"}), None),
      ExecutionData::new_json(json!({"id": 5, "name": "Eve"}), None),
    ];

    let config = LoopConfig {
      mode: LoopMode::Batch,
      iterations: None,
      batch_size: Some(2),
      condition: None,
      max_iterations: Some(3),
      include_index: false,
      parallel: false,
    };

    let result = process_batch(&input_items, &config, &"test".into()).unwrap();

    assert_eq!(result.len(), 3);

    // 第一批
    assert_eq!(result[0].json()["batch_index"], 0);
    assert_eq!(result[0].json()["items"].as_array().unwrap().len(), 2);
    assert_eq!(result[0].json()["items"][0]["id"], 1);
    assert_eq!(result[0].json()["items"][1]["id"], 2);

    // 第二批
    assert_eq!(result[1].json()["batch_index"], 1);
    assert_eq!(result[1].json()["items"].as_array().unwrap().len(), 2);
    assert_eq!(result[1].json()["items"][0]["id"], 3);
    assert_eq!(result[1].json()["items"][1]["id"], 4);

    // 第三批
    assert_eq!(result[2].json()["batch_index"], 2);
    assert_eq!(result[2].json()["items"].as_array().unwrap().len(), 1);
    assert_eq!(result[2].json()["items"][0]["id"], 5);
  }

  #[test]
  fn test_evaluate_condition() {
    // 测试布尔值条件
    let data_true = json!({"enabled": true});
    assert!(evaluate_condition(&data_true, "enabled"));

    let data_false = json!({"enabled": false});
    assert!(!evaluate_condition(&data_false, "enabled"));

    // 测试存在性条件
    let data_with_field = json!({"name": "Alice"});
    assert!(evaluate_condition(&data_with_field, "name"));

    let data_without_field = json!({"age": 25});
    assert!(!evaluate_condition(&data_without_field, "name"));

    // 测试字符串条件
    let data_with_string = json!({"status": "active"});
    assert!(evaluate_condition(&data_with_string, "status"));

    let data_empty_string = json!({"status": ""});
    // Empty string should evaluate to false
    assert!(!evaluate_condition(&data_empty_string, "status"));

    // 测试数值条件
    let data_positive = json!({"count": 5});
    assert!(evaluate_condition(&data_positive, "count"));

    let data_zero = json!({"count": 0});
    assert!(!evaluate_condition(&data_zero, "count"));

    let data_negative = json!({"count": -5});
    assert!(!evaluate_condition(&data_negative, "count"));
  }
}
