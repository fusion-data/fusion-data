//! Set 节点工具函数

use serde_json::{Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::{OperationKind, SetOperation, ValueSourceKind};
use hetumind_core::workflow::NodeExecutionError;

/// 应用所有设置操作到数据上
pub fn apply_operations(data: &Value, operations: &[SetOperation]) -> Result<Value, NodeExecutionError> {
  let mut result = data.clone();

  for operation in operations {
    result = apply_operation(&result, operation)?;
  }

  Ok(result)
}

/// 应用单个设置操作
pub fn apply_operation(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  match operation.kind {
    OperationKind::Set => apply_set_operation(data, operation),
    OperationKind::Remove => apply_remove_operation(data, operation),
    OperationKind::Copy => apply_copy_operation(data, operation),
    OperationKind::Increment => apply_increment_operation(data, operation),
    OperationKind::Append => apply_append_operation(data, operation),
  }
}

/// 应用设置操作
fn apply_set_operation(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  let value_to_set = resolve_value(data, operation)?;
  set_nested_value(data, &operation.field_path, value_to_set)
}

/// 应用删除操作
fn apply_remove_operation(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  remove_nested_value(data, &operation.field_path)
}

/// 应用复制操作
fn apply_copy_operation(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  if let Some(source_path) = operation.value.as_ref().and_then(|v| v.as_str())
    && let Some(source_value) = get_nested_value(data, source_path)
  {
    return set_nested_value(data, &operation.field_path, source_value);
  }

  // 如果源路径不存在，保持原数据不变
  Ok(data.clone())
}

/// 应用增加操作（仅适用于数值）
fn apply_increment_operation(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  let increment_value = resolve_value(data, operation)?;
  let increment_num = to_number(&increment_value).unwrap_or(1.0);

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_num) = to_number(&current_value)
  {
    let new_value = json!(current_num + increment_num);
    return set_nested_value(data, &operation.field_path, new_value);
  }

  // 如果字段不存在或不是数值，设置为增量值
  set_nested_value(data, &operation.field_path, increment_value)
}

/// 应用追加操作（仅适用于数组）
fn apply_append_operation(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  let value_to_append = resolve_value(data, operation)?;

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_array) = current_value.as_array()
  {
    let mut new_array = current_array.clone();
    new_array.push(value_to_append);
    return set_nested_value(data, &operation.field_path, json!(new_array));
  }

  // 如果字段不存在或不是数组，创建新数组
  set_nested_value(data, &operation.field_path, json!([value_to_append]))
}

/// 解析操作值
fn resolve_value(data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
  match operation.value_source {
    ValueSourceKind::Static => Ok(operation.value.clone().unwrap_or(Value::Null)),
    ValueSourceKind::Expression => {
      if let Some(expr) = operation.value.as_ref().and_then(|v| v.as_str())
        && let Some(path) = expr.strip_prefix("$.")
      {
        // 简单的 JSON Path 支持
        return Ok(get_nested_value(data, path).unwrap_or(Value::Null));
      }
      Ok(operation.value.clone().unwrap_or(Value::Null))
    }
    ValueSourceKind::CurrentTimestamp => {
      let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
      Ok(json!(timestamp))
    }
    ValueSourceKind::Random => {
      let mut hasher = DefaultHasher::new();
      std::time::SystemTime::now().hash(&mut hasher);
      let random_num = hasher.finish() as f64 / u64::MAX as f64;
      Ok(json!(random_num))
    }
  }
}

/// 获取嵌套值
pub fn get_nested_value(data: &Value, path: &str) -> Option<Value> {
  let parts: Vec<&str> = path.split('.').collect();
  let mut current = data;

  for part in parts {
    match current {
      Value::Object(obj) => {
        current = obj.get(part)?;
      }
      _ => return None,
    }
  }

  Some(current.clone())
}

/// 设置嵌套值
pub fn set_nested_value(data: &Value, path: &str, value: Value) -> Result<Value, NodeExecutionError> {
  let parts: Vec<&str> = path.split('.').collect();
  if parts.is_empty() {
    return Ok(value);
  }

  let mut result = data.clone();

  // 确保根对象是一个对象
  if !result.is_object() {
    result = json!({});
  }

  set_nested_value_recursive(&mut result, &parts, 0, value)?;
  Ok(result)
}

/// 递归设置嵌套值
fn set_nested_value_recursive(
  current: &mut Value,
  parts: &[&str],
  index: usize,
  value: Value,
) -> Result<(), NodeExecutionError> {
  if index >= parts.len() {
    return Ok(());
  }

  let part = parts[index];

  // 确保当前值是一个对象
  if !current.is_object() {
    *current = json!({});
  }

  if index == parts.len() - 1 {
    // 最后一级，直接设置值
    if let Some(obj) = current.as_object_mut() {
      obj.insert(part.to_string(), value);
    }
  } else {
    // 中间层级，递归处理
    if let Some(obj) = current.as_object_mut() {
      if !obj.contains_key(part) {
        obj.insert(part.to_string(), json!({}));
      }
      if let Some(next_value) = obj.get_mut(part) {
        set_nested_value_recursive(next_value, parts, index + 1, value)?;
      }
    }
  }

  Ok(())
}

/// 删除嵌套值
pub fn remove_nested_value(data: &Value, path: &str) -> Result<Value, NodeExecutionError> {
  let parts: Vec<&str> = path.split('.').collect();
  if parts.is_empty() {
    return Ok(data.clone());
  }

  let mut result = data.clone();
  remove_nested_value_recursive(&mut result, &parts, 0)?;
  Ok(result)
}

/// 递归删除嵌套值
fn remove_nested_value_recursive(current: &mut Value, parts: &[&str], index: usize) -> Result<(), NodeExecutionError> {
  if index >= parts.len() {
    return Ok(());
  }

  let part = parts[index];

  if let Some(obj) = current.as_object_mut() {
    if index == parts.len() - 1 {
      // 最后一级，删除字段
      obj.remove(part);
    } else if let Some(next_value) = obj.get_mut(part) {
      // 中间层级，递归处理
      remove_nested_value_recursive(next_value, parts, index + 1)?;
    }
  }

  Ok(())
}

/// 转换为数值
pub fn to_number(value: &Value) -> Option<f64> {
  match value {
    Value::Number(n) => n.as_f64(),
    Value::String(s) => s.parse().ok(),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_nested_value() {
    let data = json!({
      "user": {
        "name": "John",
        "profile": {
          "age": 30
        }
      }
    });

    assert_eq!(get_nested_value(&data, "user.name"), Some(json!("John")));
    assert_eq!(get_nested_value(&data, "user.profile.age"), Some(json!(30)));
    assert_eq!(get_nested_value(&data, "user.nonexistent"), None);
  }

  #[test]
  fn test_set_nested_value() {
    let data = json!({"user": {"name": "John"}});

    let result = set_nested_value(&data, "user.age", json!(25)).unwrap();
    assert_eq!(result["user"]["age"], json!(25));

    // 设置嵌套路径
    let result = set_nested_value(&data, "user.profile.email", json!("john@example.com")).unwrap();
    assert_eq!(result["user"]["profile"]["email"], json!("john@example.com"));
  }

  #[test]
  fn test_remove_nested_value() {
    let data = json!({
      "user": {
        "name": "John",
        "temp": "data"
      }
    });

    let result = remove_nested_value(&data, "user.temp").unwrap();
    assert!(!result["user"].as_object().unwrap().contains_key("temp"));
    assert!(result["user"].as_object().unwrap().contains_key("name"));
  }

  #[test]
  fn test_to_number() {
    assert_eq!(to_number(&json!(42)), Some(42.0));
    assert_eq!(to_number(&json!("3.17")), Some(3.17f64));
    assert_eq!(to_number(&json!("hello")), None);
    assert_eq!(to_number(&json!(true)), None);
  }

  #[test]
  fn test_apply_set_operation() {
    let data = json!({"user": {"name": "John"}});
    let operation = SetOperation {
      field_path: "user.age".to_string(),
      kind: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: Some(json!(25)),
      keep: None,
    };

    let result = apply_set_operation(&data, &operation).unwrap();
    assert_eq!(result["user"]["age"], json!(25));
  }

  #[test]
  fn test_apply_increment_operation() {
    let data = json!({"counter": 10});
    let operation = SetOperation {
      field_path: "counter".to_string(),
      kind: OperationKind::Increment,
      value_source: ValueSourceKind::Static,
      value: Some(json!(5)),
      keep: None,
    };

    let result = apply_increment_operation(&data, &operation).unwrap();
    assert_eq!(result["counter"], json!(15));
  }

  #[test]
  fn test_apply_append_operation() {
    let data = json!({"items": [1, 2, 3]});
    let operation = SetOperation {
      field_path: "items".to_string(),
      kind: OperationKind::Append,
      value_source: ValueSourceKind::Static,
      value: Some(json!(4)),
      keep: None,
    };

    let result = apply_append_operation(&data, &operation).unwrap();
    assert_eq!(result["items"], json!([1, 2, 3, 4]));
  }
}
