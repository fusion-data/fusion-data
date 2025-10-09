//! Merge 节点工具函数

use serde_json::{Map, Value, json};
use std::collections::HashMap;

use super::{MergeConfig, MergeMode};
use hetumind_core::workflow::{DataSource, ExecutionData, NodeExecutionError, NodeName};

/// 应用合并操作到数据上
pub fn merge_data(
  input_items: &[ExecutionData],
  config: &MergeConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  match config.mode {
    MergeMode::Append => merge_append(input_items, current_node_name),
    MergeMode::MergeByKey => merge_by_key(input_items, &config.merge_key, current_node_name),
    MergeMode::MergeByIndex => merge_by_index(input_items, current_node_name),
    MergeMode::WaitForAll => merge_wait_for_all(input_items, config, current_node_name),
  }
}

/// 简单追加合并
fn merge_append(
  input_items: &[ExecutionData],
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let mut result = Vec::new();

  for (index, data) in input_items.iter().enumerate() {
    result.push(ExecutionData::new_json(
      data.json().clone(),
      Some(DataSource {
        node_name: current_node_name.clone(),
        output_port: hetumind_core::workflow::ConnectionKind::Main,
        output_index: index,
      }),
    ));
  }

  Ok(result)
}

/// 按键合并
fn merge_by_key(
  input_items: &[ExecutionData],
  merge_key: &Option<String>,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let key_field = merge_key.as_ref().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: "Merge key is required for MergeByKey mode".to_string(),
  })?;

  let mut grouped_data: HashMap<String, Vec<&ExecutionData>> = HashMap::default();

  // 按键分组数据
  for data in input_items {
    let key_value = extract_key_value(data.json(), key_field);
    grouped_data.entry(key_value).or_default().push(data);
  }

  let mut result = Vec::new();
  let mut output_index = 0;

  // 合并每个分组的数据
  for (_key, group) in grouped_data {
    if group.is_empty() {
      continue;
    }

    // 合并同一键的所有数据
    let merged_json = merge_json_objects(group.iter().map(|d| d.json()).collect())?;

    result.push(ExecutionData::new_json(
      merged_json,
      Some(DataSource {
        node_name: current_node_name.clone(),
        output_port: hetumind_core::workflow::ConnectionKind::Main,
        output_index,
      }),
    ));

    output_index += 1;
  }

  Ok(result)
}

/// 按索引合并
fn merge_by_index(
  input_items: &[ExecutionData],
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let mut grouped_data: HashMap<String, Vec<&ExecutionData>> = HashMap::default();

  // 按索引分组数据
  for data in input_items {
    let index = data.source().map(|s| s.output_index.to_string()).unwrap_or_else(|| "0".to_string());
    grouped_data.entry(index).or_default().push(data);
  }

  let mut result = Vec::new();
  let mut output_index = 0;

  // 按索引顺序处理
  let mut indices: Vec<_> = grouped_data.keys().collect();
  indices.sort();

  for index in indices {
    if let Some(group) = grouped_data.get(index) {
      if group.is_empty() {
        continue;
      }

      // 合并同一索引的所有数据
      let merged_json = merge_json_objects(group.iter().map(|d| d.json()).collect())?;

      result.push(ExecutionData::new_json(
        merged_json,
        Some(DataSource {
          node_name: current_node_name.clone(),
          output_port: hetumind_core::workflow::ConnectionKind::Main,
          output_index,
        }),
      ));

      output_index += 1;
    }
  }

  Ok(result)
}

/// 等待全部输入完成后合并
fn merge_wait_for_all(
  input_items: &[ExecutionData],
  config: &MergeConfig,
  current_node_name: &NodeName,
) -> Result<Vec<ExecutionData>, NodeExecutionError> {
  let expected_ports = config.input_ports.unwrap_or(2);

  // 检查是否有足够的输入分支数据
  let mut source_ports = std::collections::HashSet::new();
  for data in input_items {
    if let Some(source) = data.source() {
      source_ports.insert(&source.output_port);
    }
  }

  if source_ports.len() < expected_ports {
    log::warn!("WaitForAll 模式: 期望 {} 个输入分支，实际收到 {} 个", expected_ports, source_ports.len());
  }

  // 使用简单追加模式合并所有数据
  merge_append(input_items, current_node_name)
}

/// 提取键值
fn extract_key_value(data: &Value, key_field: &str) -> String {
  match data.get(key_field) {
    Some(Value::String(s)) => s.clone(),
    Some(value) => value.to_string(),
    None => "null".to_string(),
  }
}

/// 合并多个 JSON 对象
pub fn merge_json_objects(objects: Vec<&Value>) -> Result<Value, NodeExecutionError> {
  if objects.is_empty() {
    return Ok(Value::Null);
  }

  if objects.len() == 1 {
    return Ok(objects[0].clone());
  }

  let mut merged = Map::new();

  for obj in objects {
    match obj {
      Value::Object(map) => {
        for (key, value) in map {
          // 如果键已存在，尝试合并值
          if let Some(existing_value) = merged.get(key) {
            merged.insert(key.clone(), merge_values(existing_value, value)?);
          } else {
            merged.insert(key.clone(), value.clone());
          }
        }
      }
      _ => {
        // 非对象类型，使用 "_value" 作为键
        merged.insert("_value".to_string(), obj.clone());
      }
    }
  }

  Ok(Value::Object(merged))
}

/// 合并两个值
fn merge_values(value1: &Value, value2: &Value) -> Result<Value, NodeExecutionError> {
  match (value1, value2) {
    // 如果都是数组，合并数组
    (Value::Array(arr1), Value::Array(arr2)) => {
      let mut merged = arr1.clone();
      merged.extend(arr2.clone());
      Ok(Value::Array(merged))
    }
    // 如果都是对象，递归合并
    (Value::Object(_), Value::Object(_)) => merge_json_objects(vec![value1, value2]),
    // 其他情况，value2 覆盖 value1
    _ => Ok(value2.clone()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_merge_append() {
    let input_items = vec![
      ExecutionData::new_json(json!({"name": "Alice", "age": 25}), None),
      ExecutionData::new_json(json!({"name": "Bob", "age": 30}), None),
    ];

    let result = merge_append(&input_items, &"test".into()).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].json()["name"], "Alice");
    assert_eq!(result[1].json()["name"], "Bob");
  }

  #[test]
  fn test_merge_by_key() {
    let input_items = vec![
      ExecutionData::new_json(json!({"id": "1", "name": "Alice", "age": 25}), None),
      ExecutionData::new_json(json!({"id": "1", "city": "New York"}), None),
      ExecutionData::new_json(json!({"id": "2", "name": "Bob", "age": 30}), None),
    ];

    let merge_key = Some("id".to_string());
    let result = merge_by_key(&input_items, &merge_key, &"test".into()).unwrap();

    assert_eq!(result.len(), 2);

    // 查找 id=1 的合并结果
    let alice_record = result.iter().find(|r| r.json()["id"] == "1").unwrap();
    assert_eq!(alice_record.json()["name"], "Alice");
    assert_eq!(alice_record.json()["city"], "New York");
  }

  #[test]
  fn test_merge_by_index() {
    let input_items = vec![
      ExecutionData::new_json(json!({"name": "Alice"}), None),
      ExecutionData::new_json(json!({"age": 25}), None),
      ExecutionData::new_json(json!({"name": "Bob"}), None),
    ];

    // 设置索引
    let mut items_with_index = input_items;
    items_with_index[0] = ExecutionData::new_json(json!({"name": "Alice"}), None);
    items_with_index[1] = ExecutionData::new_json(json!({"age": 25}), None);

    let result = merge_by_index(&items_with_index, &"test".into()).unwrap();
    assert_eq!(result.len(), 2);
  }

  #[test]
  fn test_merge_json_objects() {
    let obj1 = json!({"name": "Alice", "age": 25});
    let obj2 = json!({"age": 26, "city": "New York"});

    let result = merge_json_objects(vec![&obj1, &obj2]).unwrap();

    assert_eq!(result["name"], "Alice");
    assert_eq!(result["age"], 26); // obj2 的值覆盖了 obj1
    assert_eq!(result["city"], "New York");
  }

  #[test]
  fn test_extract_key_value() {
    let data = json!({"id": "123", "name": "Alice"});
    assert_eq!(extract_key_value(&data, "id"), "123");
    assert_eq!(extract_key_value(&data, "name"), "Alice");
    assert_eq!(extract_key_value(&data, "nonexistent"), "null");

    let data_with_number = json!({"id": 123, "name": "Alice"});
    assert_eq!(extract_key_value(&data_with_number, "id"), "123");
  }

  #[test]
  fn test_merge_values() {
    // 测试数组合并
    let arr1 = json!([1, 2, 3]);
    let arr2 = json!([4, 5, 6]);
    let result = merge_values(&arr1, &arr2).unwrap();
    assert_eq!(result, json!([1, 2, 3, 4, 5, 6]));

    // 测试对象合并
    let obj1 = json!({"a": 1});
    let obj2 = json!({"b": 2});
    let result = merge_values(&obj1, &obj2).unwrap();
    assert_eq!(result, json!({"a": 1, "b": 2}));

    // 测试覆盖
    let val1 = json!("old");
    let val2 = json!("new");
    let result = merge_values(&val1, &val2).unwrap();
    assert_eq!(result, json!("new"));
  }
}
