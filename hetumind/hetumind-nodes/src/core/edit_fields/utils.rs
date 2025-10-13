//! Edit Fields 节点工具函数
//!
//! 提供字段操作、类型转换、表达式处理等核心功能。
//! 支持 Manual Mapping 和 JSON 两种操作模式。

use chrono::Utc;
use regex::Regex;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{BinaryDataMode, EditFieldsOptions, FieldOperation, IncludeMode, OperationKind, ValueSourceKind};
use hetumind_core::workflow::NodeExecutionError;

/// 应用字段操作到数据
pub fn apply_field_operations(
  data: &Value,
  operations: &[FieldOperation],
  include_mode: &IncludeMode,
  selected_fields: &Option<Vec<String>>,
  options: &EditFieldsOptions,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let mut result = data.clone();

  // 应用所有字段操作
  for operation in operations {
    result = apply_field_operation(&result, operation, options, item_index)?;
  }

  // 应用输出包含模式
  result = apply_include_mode(&result, include_mode, selected_fields, data)?;

  // 处理二进制数据
  if let Some(binary_mode) = &options.binary_data_mode {
    result = handle_binary_data(&result, binary_mode);
  }

  Ok(result)
}

/// 应用 JSON 模板
pub fn apply_json_template(
  data: &Value,
  template: &str,
  use_expressions: bool,
  _options: &EditFieldsOptions,
  _item_index: usize,
) -> Result<Value, NodeExecutionError> {
  if !use_expressions {
    // 如果不使用表达式，直接解析 JSON
    return match serde_json::from_str(template) {
      Ok(value) => Ok(value),
      Err(_) => Err(NodeExecutionError::DataProcessingError { message: "Invalid JSON template".to_string() }),
    };
  }

  // 处理带表达式的模板
  let mut result = template.to_string();

  // 替换简单变量引用 {{field_name}}
  let variable_regex = Regex::new(r"\{\{([^}]+)\}\}").unwrap();
  let captures: Vec<_> = variable_regex.captures_iter(template).collect();

  for cap in captures {
    if let Some(var_match) = cap.get(1) {
      let var_path = var_match.as_str().trim();
      let replacement = get_nested_value(data, var_path).unwrap_or(Value::Null);
      let replacement_str = match replacement {
        Value::String(s) => s,
        _ => replacement.to_string(),
      };

      result = result.replace(&cap[0], &replacement_str);
    }
  }

  // 尝试解析为 JSON
  match serde_json::from_str(&result) {
    Ok(value) => Ok(value),
    Err(_) => {
      // 如果解析失败，返回字符串
      Ok(Value::String(result))
    }
  }
}

/// 应用单个字段操作
fn apply_field_operation(
  data: &Value,
  operation: &FieldOperation,
  options: &EditFieldsOptions,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let ignore_errors = operation.ignore_conversion_error.unwrap_or(options.ignore_conversion_errors.unwrap_or(false));

  let result = match operation.operation {
    OperationKind::Set => apply_set_operation(data, operation, item_index),
    OperationKind::Remove => apply_remove_operation(data, operation, item_index),
    OperationKind::Copy => apply_copy_operation(data, operation, item_index),
    OperationKind::Increment => apply_increment_operation(data, operation, item_index),
    OperationKind::Append => apply_append_operation(data, operation, item_index),
    OperationKind::Prepend => apply_prepend_operation(data, operation, item_index),
    OperationKind::Multiply => apply_multiply_operation(data, operation, item_index),
    OperationKind::Replace => apply_replace_operation(data, operation, item_index),
    OperationKind::Split => apply_split_operation(data, operation, item_index),
    OperationKind::Join => apply_join_operation(data, operation, item_index),
  };

  match result {
    Ok(value) => Ok(value),
    Err(e) => {
      if ignore_errors {
        log::warn!("Ignoring field operation error: {}", e);
        Ok(data.clone())
      } else {
        Err(e)
      }
    }
  }
}

/// 应用设置操作
fn apply_set_operation(
  data: &Value,
  operation: &FieldOperation,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let value_to_set =
    resolve_value(&operation.value_source, &operation.value, data, operation.field_type.as_ref(), item_index)?;
  set_nested_value(data, &operation.field_path, value_to_set)
}

/// 应用删除操作
fn apply_remove_operation(
  data: &Value,
  operation: &FieldOperation,
  _item_index: usize,
) -> Result<Value, NodeExecutionError> {
  remove_nested_value(data, &operation.field_path)
}

/// 应用复制操作
fn apply_copy_operation(
  data: &Value,
  operation: &FieldOperation,
  _item_index: usize,
) -> Result<Value, NodeExecutionError> {
  if let Some(source_path) = operation.value.as_ref().and_then(|v| v.as_str())
    && let Some(source_value) = get_nested_value(data, source_path)
  {
    return set_nested_value(data, &operation.field_path, source_value);
  }
  Ok(data.clone())
}

/// 应用增加操作
fn apply_increment_operation(
  data: &Value,
  operation: &FieldOperation,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let increment_value =
    resolve_value(&operation.value_source, &operation.value, data, operation.field_type.as_ref(), item_index)?;
  let increment_num = to_number(&increment_value).unwrap_or(1.0);

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_num) = to_number(&current_value)
  {
    let new_value = json!(current_num + increment_num);
    return set_nested_value(data, &operation.field_path, new_value);
  }

  set_nested_value(data, &operation.field_path, increment_value)
}

/// 应用追加操作
fn apply_append_operation(
  data: &Value,
  operation: &FieldOperation,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let value_to_append =
    resolve_value(&operation.value_source, &operation.value, data, operation.field_type.as_ref(), item_index)?;

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_array) = current_value.as_array()
  {
    let mut new_array = current_array.clone();
    new_array.push(value_to_append);
    return set_nested_value(data, &operation.field_path, json!(new_array));
  }

  set_nested_value(data, &operation.field_path, json!([value_to_append]))
}

/// 应用前置操作
fn apply_prepend_operation(
  data: &Value,
  operation: &FieldOperation,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let value_to_prepend =
    resolve_value(&operation.value_source, &operation.value, data, operation.field_type.as_ref(), item_index)?;

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_array) = current_value.as_array()
  {
    let mut new_array = current_array.clone();
    new_array.insert(0, value_to_prepend);
    return set_nested_value(data, &operation.field_path, json!(new_array));
  }

  set_nested_value(data, &operation.field_path, json!([value_to_prepend]))
}

/// 应用乘法操作
fn apply_multiply_operation(
  data: &Value,
  operation: &FieldOperation,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let multiply_value =
    resolve_value(&operation.value_source, &operation.value, data, operation.field_type.as_ref(), item_index)?;
  let multiply_num = to_number(&multiply_value).unwrap_or(1.0);

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_num) = to_number(&current_value)
  {
    let new_value = json!(current_num * multiply_num);
    return set_nested_value(data, &operation.field_path, new_value);
  }

  set_nested_value(data, &operation.field_path, multiply_value)
}

/// 应用替换操作
fn apply_replace_operation(
  data: &Value,
  operation: &FieldOperation,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let replacement_value =
    resolve_value(&operation.value_source, &operation.value, data, operation.field_type.as_ref(), item_index)?;

  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_str) = current_value.as_str()
  {
    let search_string = operation
      .operation_params
      .as_ref()
      .and_then(|p| p.get("search_string"))
      .and_then(|s| s.as_str())
      .unwrap_or("");

    let replace_string = replacement_value.as_str().unwrap_or("");

    let case_sensitive = operation
      .operation_params
      .as_ref()
      .and_then(|p| p.get("case_sensitive"))
      .and_then(|c| c.as_bool())
      .unwrap_or(true);

    let result_str = if case_sensitive {
      current_str.replace(search_string, replace_string)
    } else {
      let regex = Regex::new(&regex::escape(search_string)).unwrap();
      regex.replace_all(current_str, replace_string).to_string()
    };

    return set_nested_value(data, &operation.field_path, json!(result_str));
  }

  set_nested_value(data, &operation.field_path, replacement_value)
}

/// 应用分割操作
fn apply_split_operation(
  data: &Value,
  operation: &FieldOperation,
  _item_index: usize,
) -> Result<Value, NodeExecutionError> {
  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_str) = current_value.as_str()
  {
    let separator = operation
      .operation_params
      .as_ref()
      .and_then(|p| p.get("separator"))
      .and_then(|s| s.as_str())
      .unwrap_or(",");

    let max_splits = operation
      .operation_params
      .as_ref()
      .and_then(|p| p.get("max_splits"))
      .and_then(|m| m.as_u64())
      .map(|m| m as usize);

    let result_array: Vec<&str> = if let Some(max) = max_splits {
      current_str.splitn(max + 1, separator).collect()
    } else {
      current_str.split(separator).collect()
    };

    let result_json: Vec<Value> = result_array.into_iter().map(|s| json!(s)).collect();
    return set_nested_value(data, &operation.field_path, json!(result_json));
  }

  Ok(data.clone())
}

/// 应用连接操作
fn apply_join_operation(
  data: &Value,
  operation: &FieldOperation,
  _item_index: usize,
) -> Result<Value, NodeExecutionError> {
  if let Some(current_value) = get_nested_value(data, &operation.field_path)
    && let Some(current_array) = current_value.as_array()
  {
    let separator = operation
      .operation_params
      .as_ref()
      .and_then(|p| p.get("separator"))
      .and_then(|s| s.as_str())
      .unwrap_or(",");

    let string_values: Vec<String> =
      current_array.iter().map(|v| v.as_str().unwrap_or(&v.to_string()).to_string()).collect();

    let result_str = string_values.join(separator);
    return set_nested_value(data, &operation.field_path, json!(result_str));
  }

  Ok(data.clone())
}

/// 解析值来源
fn resolve_value(
  value_source: &ValueSourceKind,
  value: &Option<Value>,
  data: &Value,
  field_type: Option<&super::FieldType>,
  item_index: usize,
) -> Result<Value, NodeExecutionError> {
  let resolved_value = match value_source {
    ValueSourceKind::Static => value.clone().unwrap_or(Value::Null),
    ValueSourceKind::Expression => {
      if let Some(expr) = value.as_ref().and_then(|v| v.as_str()) {
        if let Some(path) = expr.strip_prefix("$.") {
          get_nested_value(data, path).unwrap_or(Value::Null)
        } else {
          // 处理更复杂的表达式
          resolve_expression(expr, data, item_index)?
        }
      } else {
        Value::Null
      }
    }
    ValueSourceKind::CurrentTimestamp => {
      let now = Utc::now();
      json!(now.timestamp())
    }
    ValueSourceKind::Random => {
      use rand::Rng;
      let mut rng = rand::rng();
      let random_value: f64 = rng.random();
      json!(random_value)
    }
    ValueSourceKind::Uuid => {
      let uuid = Uuid::new_v4().to_string();
      json!(uuid)
    }
    ValueSourceKind::JsonPath => {
      if let Some(path_expr) = value.as_ref().and_then(|v| v.as_str()) {
        get_nested_value(data, path_expr).unwrap_or(Value::Null)
      } else {
        Value::Null
      }
    }
    ValueSourceKind::EnvironmentVariable => {
      if let Some(var_name) = value.as_ref().and_then(|v| v.as_str()) {
        if let Ok(var_value) = std::env::var(var_name) { json!(var_value) } else { Value::Null }
      } else {
        Value::Null
      }
    }
  };

  // 应用类型转换
  if let Some(field_type) = field_type { convert_type(&resolved_value, field_type) } else { Ok(resolved_value) }
}

/// 解析表达式
fn resolve_expression(expr: &str, data: &Value, item_index: usize) -> Result<Value, NodeExecutionError> {
  // 支持一些内置表达式
  match expr {
    "$index" => Ok(json!(item_index)),
    "$now" => Ok(json!(Utc::now().timestamp())),
    "$today" => Ok(json!(Utc::now().format("%Y-%m-%d").to_string())),
    _ if expr.starts_with("$.") => {
      let path = &expr[2..];
      Ok(get_nested_value(data, path).unwrap_or(Value::Null))
    }
    _ => Ok(Value::String(expr.to_string())),
  }
}

/// 类型转换
fn convert_type(value: &Value, field_type: &super::FieldType) -> Result<Value, NodeExecutionError> {
  match field_type {
    super::FieldType::String => Ok(json!(value.to_string())),
    super::FieldType::Number => match to_number(value) {
      Some(num) => Ok(json!(num)),
      None => Err(NodeExecutionError::DataProcessingError { message: format!("Cannot convert '{}' to number", value) }),
    },
    super::FieldType::Boolean => match value {
      Value::Bool(b) => Ok(json!(b)),
      Value::String(s) => match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(json!(true)),
        "false" | "0" | "no" | "off" => Ok(json!(false)),
        _ => Err(NodeExecutionError::DataProcessingError { message: format!("Cannot convert '{}' to boolean", s) }),
      },
      Value::Number(n) => Ok(json!(n.as_f64().unwrap_or(0.0) != 0.0)),
      _ => Err(NodeExecutionError::DataProcessingError { message: format!("Cannot convert '{}' to boolean", value) }),
    },
    super::FieldType::Array => match value {
      Value::Array(_) => Ok(value.clone()),
      _ => Ok(json!([value.clone()])),
    },
    super::FieldType::Object => match value {
      Value::Object(_) => Ok(value.clone()),
      _ => Err(NodeExecutionError::DataProcessingError { message: format!("Cannot convert '{}' to object", value) }),
    },
    super::FieldType::Auto => Ok(value.clone()),
  }
}

/// 应用包含模式
fn apply_include_mode(
  data: &Value,
  include_mode: &IncludeMode,
  selected_fields: &Option<Vec<String>>,
  _original_data: &Value,
) -> Result<Value, NodeExecutionError> {
  if !data.is_object() {
    return Ok(data.clone());
  }

  let data_obj = data.as_object().unwrap();
  match include_mode {
    IncludeMode::All => Ok(data.clone()),
    IncludeMode::None => Ok(json!({})),
    IncludeMode::Selected => {
      if let Some(fields) = selected_fields {
        let mut result = json!({});
        if let Some(result_obj) = result.as_object_mut() {
          for field in fields {
            if let Some(value) = data_obj.get(field) {
              result_obj.insert(field.clone(), value.clone());
            }
          }
        }
        Ok(result)
      } else {
        Ok(data.clone())
      }
    }
    IncludeMode::Except => {
      if let Some(fields) = selected_fields {
        let mut result = data.clone();
        if let Some(result_obj) = result.as_object_mut() {
          for field in fields {
            result_obj.remove(field);
          }
        }
        Ok(result)
      } else {
        Ok(data.clone())
      }
    }
  }
}

/// 处理二进制数据
fn handle_binary_data(data: &Value, mode: &BinaryDataMode) -> Value {
  match mode {
    BinaryDataMode::Include => data.clone(),
    BinaryDataMode::Strip => {
      if let Some(obj) = data.as_object() {
        let mut result = obj.clone();
        result.retain(|key, _| !key.starts_with("__binary_"));
        json!(result)
      } else {
        data.clone()
      }
    }
    BinaryDataMode::Auto => {
      // 自动检测是否有二进制数据
      if has_binary_data(data) { handle_binary_data(data, &BinaryDataMode::Strip) } else { data.clone() }
    }
  }
}

/// 检测是否有二进制数据
fn has_binary_data(data: &Value) -> bool {
  if let Some(obj) = data.as_object() { obj.keys().any(|key| key.starts_with("__binary_")) } else { false }
}

/// 获取嵌套值
pub fn get_nested_value(data: &Value, path: &str) -> Option<Value> {
  if path.is_empty() {
    return Some(data.clone());
  }

  let parts: Vec<&str> = path.split('.').collect();
  let mut current = data;

  for part in parts {
    match current {
      Value::Object(obj) => {
        current = obj.get(part)?;
      }
      Value::Array(arr) => {
        if let Ok(index) = part.parse::<usize>() {
          current = arr.get(index)?;
        } else {
          return None;
        }
      }
      _ => return None,
    }
  }

  Some(current.clone())
}

/// 设置嵌套值
pub fn set_nested_value(data: &Value, path: &str, value: Value) -> Result<Value, NodeExecutionError> {
  if path.is_empty() {
    return Ok(value);
  }

  let parts: Vec<&str> = path.split('.').collect();
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
  if path.is_empty() {
    return Ok(data.clone());
  }

  let parts: Vec<&str> = path.split('.').collect();
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
    Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
    Value::Null => Some(0.0),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::super::{BinaryDataMode, EditFieldsOptions, FieldType, IncludeMode, OperationKind, ValueSourceKind};
  use super::*;

  #[test]
  fn test_get_nested_value() {
    let data = json!({
      "user": {
        "name": "John",
        "profile": {
          "age": 30,
          "hobbies": ["reading", "coding"]
        }
      }
    });

    assert_eq!(get_nested_value(&data, "user.name"), Some(json!("John")));
    assert_eq!(get_nested_value(&data, "user.profile.age"), Some(json!(30)));
    assert_eq!(get_nested_value(&data, "user.profile.hobbies.1"), Some(json!("coding")));
    assert_eq!(get_nested_value(&data, "user.nonexistent"), None);
  }

  #[test]
  fn test_set_nested_value() {
    let data = json!({"user": {"name": "John"}});

    let result = set_nested_value(&data, "user.age", json!(25)).unwrap();
    assert_eq!(result["user"]["age"], json!(25));

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
    assert_eq!(to_number(&json!("3.17")), Some(3.17));
    assert_eq!(to_number(&json!("hello")), None);
    assert_eq!(to_number(&json!(true)), Some(1.0));
    assert_eq!(to_number(&json!(false)), Some(0.0));
    assert_eq!(to_number(&json!(null)), Some(0.0));
  }

  #[test]
  fn test_apply_set_operation() {
    let data = json!({"user": {"name": "John"}});
    let operation = FieldOperation {
      field_path: "user.age".to_string(),
      operation: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: Some(json!(25)),
      field_type: Some(FieldType::Number),
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["user"]["age"], json!(25.0));
  }

  #[test]
  fn test_apply_increment_operation() {
    let data = json!({"counter": 10});
    let operation = FieldOperation {
      field_path: "counter".to_string(),
      operation: OperationKind::Increment,
      value_source: ValueSourceKind::Static,
      value: Some(json!(5)),
      field_type: Some(FieldType::Number),
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["counter"], json!(15.0));
  }

  #[test]
  fn test_apply_append_operation() {
    let data = json!({"items": [1, 2, 3]});
    let operation = FieldOperation {
      field_path: "items".to_string(),
      operation: OperationKind::Append,
      value_source: ValueSourceKind::Static,
      value: Some(json!(4)),
      field_type: None,
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["items"], json!([1, 2, 3, 4]));
  }

  #[test]
  fn test_apply_prepend_operation() {
    let data = json!({"items": [2, 3, 4]});
    let operation = FieldOperation {
      field_path: "items".to_string(),
      operation: OperationKind::Prepend,
      value_source: ValueSourceKind::Static,
      value: Some(json!(1)),
      field_type: None,
      operation_params: None,
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["items"], json!([1, 2, 3, 4]));
  }

  #[test]
  fn test_apply_replace_operation() {
    let data = json!({"message": "Hello world!"});
    let operation = FieldOperation {
      field_path: "message".to_string(),
      operation: OperationKind::Replace,
      value_source: ValueSourceKind::Static,
      value: Some(json!("Hi")),
      field_type: None,
      operation_params: Some(json!({
        "search_string": "Hello",
        "case_sensitive": true
      })),
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["message"], json!("Hi world!"));
  }

  #[test]
  fn test_apply_split_operation() {
    let data = json!({"text": "apple,banana,cherry"});
    let operation = FieldOperation {
      field_path: "text".to_string(),
      operation: OperationKind::Split,
      value_source: ValueSourceKind::Static,
      value: None,
      field_type: None,
      operation_params: Some(json!({
        "separator": ","
      })),
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["text"], json!(["apple", "banana", "cherry"]));
  }

  #[test]
  fn test_apply_join_operation() {
    let data = json!({"items": ["apple", "banana", "cherry"]});
    let operation = FieldOperation {
      field_path: "items".to_string(),
      operation: OperationKind::Join,
      value_source: ValueSourceKind::Static,
      value: None,
      field_type: None,
      operation_params: Some(json!({
        "separator": "; "
      })),
      keep_original_type: None,
      ignore_conversion_error: None,
    };

    let options = EditFieldsOptions::default();
    let result = apply_field_operation(&data, &operation, &options, 0).unwrap();
    assert_eq!(result["items"], json!("apple; banana; cherry"));
  }

  #[test]
  fn test_include_mode_selected() {
    let data = json!({
      "name": "John",
      "age": 30,
      "email": "john@example.com"
    });

    let result =
      apply_include_mode(&data, &IncludeMode::Selected, &Some(vec!["name".to_string(), "email".to_string()]), &data)
        .unwrap();

    assert!(result.as_object().unwrap().contains_key("name"));
    assert!(result.as_object().unwrap().contains_key("email"));
    assert!(!result.as_object().unwrap().contains_key("age"));
  }

  #[test]
  fn test_include_mode_except() {
    let data = json!({
      "name": "John",
      "age": 30,
      "email": "john@example.com",
      "password": "secret"
    });

    let result = apply_include_mode(&data, &IncludeMode::Except, &Some(vec!["password".to_string()]), &data).unwrap();

    assert!(result.as_object().unwrap().contains_key("name"));
    assert!(result.as_object().unwrap().contains_key("age"));
    assert!(result.as_object().unwrap().contains_key("email"));
    assert!(!result.as_object().unwrap().contains_key("password"));
  }

  #[test]
  fn test_binary_data_handling() {
    let data = json!({
      "name": "test",
      "__binary_file": "binary content"
    });

    let result = handle_binary_data(&data, &BinaryDataMode::Strip);
    assert!(result.as_object().unwrap().contains_key("name"));
    assert!(!result.as_object().unwrap().contains_key("__binary_file"));
  }

  #[test]
  fn test_type_conversion() {
    // String conversion
    let result = convert_type(&json!(42), &FieldType::String).unwrap();
    assert_eq!(result, json!("42"));

    // Number conversion
    let result = convert_type(&json!("3.178"), &FieldType::Number).unwrap();
    assert_eq!(result, json!(3.178));

    // Boolean conversion
    let result = convert_type(&json!("true"), &FieldType::Boolean).unwrap();
    assert_eq!(result, json!(true));

    let result = convert_type(&json!(1), &FieldType::Boolean).unwrap();
    assert_eq!(result, json!(true));

    // Array conversion
    let result = convert_type(&json!(42), &FieldType::Array).unwrap();
    assert_eq!(result, json!([42]));
  }

  #[test]
  fn test_resolve_value_expressions() {
    let data = json!({
      "user": {
        "name": "John",
        "age": 30
      }
    });

    // Expression
    let result = resolve_value(&ValueSourceKind::Expression, &Some(json!("$.user.name")), &data, None, 0).unwrap();
    assert_eq!(result, json!("John"));

    // Current timestamp
    let result = resolve_value(&ValueSourceKind::CurrentTimestamp, &None, &data, None, 0).unwrap();
    assert!(result.is_number());

    // UUID
    let result = resolve_value(&ValueSourceKind::Uuid, &None, &data, None, 0).unwrap();
    assert!(result.is_string());
    let uuid_str = result.as_str().unwrap();
    assert_eq!(uuid_str.len(), 36); // Standard UUID length
  }
}
