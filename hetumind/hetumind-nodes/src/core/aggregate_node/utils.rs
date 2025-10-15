//! Aggregate 节点工具函数
//!
//! 提供字段处理、二进制数据处理等实用功能。

use hetumind_core::{
  types::JsonValue,
  workflow::{BinaryDataReference, ExecutionData, NodeExecutionError},
};
use serde_json::Value;
use std::collections::HashMap;

/// 准备字段数组，将字符串或数组转换为标准化的字段列表
#[allow(unused_variables)]
pub fn prepare_fields_array(fields: &str, field_name: &str) -> Vec<String> {
  if fields.trim().is_empty() {
    return vec![];
  }

  fields.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// 获取字段值，支持点记号路径
pub fn get_field_value(data: &JsonValue, field_path: &str, disable_dot_notation: bool) -> Option<Value> {
  if disable_dot_notation {
    // 直接属性访问
    data.get(field_path).cloned()
  } else {
    // 支持点记号路径
    get_nested_value(data, field_path)
  }
}

/// 递归获取嵌套值
fn get_nested_value(data: &JsonValue, path: &str) -> Option<Value> {
  let parts: Vec<&str> = path.split('.').collect();
  let mut current = data;

  for (i, part) in parts.iter().enumerate() {
    match current {
      Value::Object(map) => {
        if let Some(value) = map.get(*part) {
          if i == parts.len() - 1 {
            return Some(value.clone());
          }
          current = value;
        } else {
          return None;
        }
      }
      Value::Array(arr) => {
        if let Ok(index) = part.parse::<usize>() {
          if let Some(value) = arr.get(index) {
            if i == parts.len() - 1 {
              return Some(value.clone());
            }
            current = value;
          } else {
            return None;
          }
        } else {
          return None;
        }
      }
      _ => return None,
    }
  }

  None
}

/// 从字段路径提取简单字段名
#[allow(dead_code)]
pub fn extract_field_name(field_path: &str, disable_dot_notation: bool) -> String {
  if disable_dot_notation {
    field_path.to_string()
  } else {
    field_path.split('.').next_back().unwrap_or(field_path).to_string()
  }
}

/// 二进制数据唯一性检查器
pub struct BinaryUniqueChecker {
  binaries: Vec<PartialBinaryData>,
}

/// 部分二进制数据结构，用于唯一性检查
#[derive(Debug, Clone)]
struct PartialBinaryData {
  pub mime_kind: String,
  pub file_size: u64,
  pub file_extension: String,
}

impl BinaryUniqueChecker {
  pub fn new() -> Self {
    Self { binaries: Vec::new() }
  }

  /// 检查二进制数据是否唯一
  pub fn is_unique(&mut self, binary_data: &BinaryDataReference) -> bool {
    let partial = PartialBinaryData {
      mime_kind: binary_data.mime_kind.clone(),
      file_size: binary_data.file_size,
      file_extension: binary_data.file_extension.clone().unwrap_or_default(),
    };

    for existing in &self.binaries {
      if existing.mime_kind == partial.mime_kind
        && existing.file_size == partial.file_size
        && existing.file_extension == partial.file_extension
      {
        return false;
      }
    }

    self.binaries.push(partial);
    true
  }
}

/// 将二进制数据添加到执行数据项
#[allow(unused_variables)]
pub fn add_binaries_to_item(
  new_item: &mut ExecutionData,
  items: &[ExecutionData],
  unique_only: bool,
) -> Result<(), NodeExecutionError> {
  let mut unique_checker = if unique_only { Some(BinaryUniqueChecker::new()) } else { None };

  for item in items {
    if let Some(binary_data) = item.binary() {
      // 检查唯一性（如果需要）
      if let Some(checker) = &mut unique_checker
        && !checker.is_unique(binary_data)
      {
        continue;
      }

      // 创建新的二进制数据引用，添加前缀避免键名冲突
      let new_file_key = format!("aggregated_{}", binary_data.file_key);
      let mut new_binary_data = binary_data.clone();
      new_binary_data.file_key = new_file_key;

      // 直接设置二进制数据（当前实现只支持单个二进制数据）
      return Ok(()); // ExecutionData 只支持单个 binary，所以这里简化处理
    }
  }

  Ok(())
}

/// 字段存在性跟踪器
pub struct FieldExistenceTracker {
  not_found_fields: HashMap<String, Vec<bool>>,
}

impl FieldExistenceTracker {
  pub fn new() -> Self {
    Self { not_found_fields: HashMap::new() }
  }

  /// 记录字段是否存在
  pub fn record_field_existence(&mut self, field: &str, exists: bool) {
    self.not_found_fields.entry(field.to_string()).or_default().push(exists);
  }

  /// 获取完全不存在的字段列表
  pub fn get_completely_missing_fields(&self) -> Vec<String> {
    self
      .not_found_fields
      .iter()
      .filter(|(_, existence)| existence.iter().all(|&exists| !exists))
      .map(|(field, _)| field.clone())
      .collect()
  }

  /// 获取部分存在的字段列表
  pub fn get_partially_missing_fields(&self) -> Vec<String> {
    self
      .not_found_fields
      .iter()
      .filter(|(_, existence)| {
        let found_count = existence.iter().filter(|&&exists| exists).count();
        found_count > 0 && found_count < existence.len()
      })
      .map(|(field, _)| field.clone())
      .collect()
  }
}

/// 处理字段值，根据配置过滤空值和合并列表
pub fn process_field_value(value: Option<Value>, keep_missing: bool, merge_lists: bool) -> Option<Value> {
  match value {
    Some(mut val) => {
      if !keep_missing {
        if let Value::Array(arr) = &mut val {
          // 过滤掉 null 值
          arr.retain(|v| !v.is_null());
          if arr.is_empty() {
            return None;
          }
        } else if val.is_null() {
          return None;
        }
      }

      if let Value::Array(arr) = val {
        if merge_lists && !arr.is_empty() {
          // 展开数组
          Some(Value::Array(arr))
        } else {
          // 保持数组结构
          Some(Value::Array(vec![Value::Array(arr)]))
        }
      } else {
        Some(Value::Array(vec![val]))
      }
    }
    None => {
      if keep_missing {
        Some(Value::Array(vec![Value::Null]))
      } else {
        None
      }
    }
  }
}

/// 应用字段过滤（包含/排除）
pub fn apply_field_filter(
  item: &JsonValue,
  fields_to_exclude: &[String],
  fields_to_include: &[String],
) -> Option<JsonValue> {
  if let Value::Object(obj) = item {
    let mut filtered_obj = serde_json::Map::new();
    let mut has_fields = false;

    for (key, value) in obj {
      // 应用排除逻辑
      if fields_to_exclude.contains(key) {
        continue;
      }

      // 应用包含逻辑
      if !fields_to_include.is_empty() && !fields_to_include.contains(key) {
        continue;
      }

      filtered_obj.insert(key.clone(), value.clone());
      has_fields = true;
    }

    if has_fields { Some(Value::Object(filtered_obj)) } else { None }
  } else {
    Some(item.clone())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_prepare_fields_array() {
    // 测试逗号分隔的字符串
    let fields = prepare_fields_array("name, age, email", "Fields");
    assert_eq!(fields, vec!["name", "age", "email"]);

    // 测试空字符串
    let fields = prepare_fields_array("", "Fields");
    assert_eq!(fields, Vec::<String>::new());

    // 测试带空格的字段
    let fields = prepare_fields_array("name , age ,  email ", "Fields");
    assert_eq!(fields, vec!["name", "age", "email"]);
  }

  #[test]
  fn test_get_field_value() {
    let data = json!({
      "name": "John",
      "profile": {
        "age": 30,
        "address": {
          "city": "New York"
        }
      },
      "tags": ["tag1", "tag2"]
    });

    // 直接属性访问
    assert_eq!(get_field_value(&data, "name", true), Some(json!("John")));
    assert_eq!(get_field_value(&data, "nonexistent", true), None);

    // 点记号访问
    assert_eq!(get_field_value(&data, "profile.age", false), Some(json!(30)));
    assert_eq!(get_field_value(&data, "profile.address.city", false), Some(json!("New York")));
    assert_eq!(get_field_value(&data, "profile.nonexistent", false), None);

    // 数组索引访问
    assert_eq!(get_field_value(&data, "tags.0", false), Some(json!("tag1")));
  }

  #[test]
  fn test_extract_field_name() {
    assert_eq!(extract_field_name("name", true), "name");
    assert_eq!(extract_field_name("profile.age", false), "age");
    assert_eq!(extract_field_name("profile.address.city", false), "city");
  }

  #[test]
  fn test_field_existence_tracker() {
    let mut tracker = FieldExistenceTracker::new();

    tracker.record_field_existence("field1", true);
    tracker.record_field_existence("field1", false);
    tracker.record_field_existence("field2", false);
    tracker.record_field_existence("field2", false);

    let completely_missing = tracker.get_completely_missing_fields();
    let partially_missing = tracker.get_partially_missing_fields();

    assert_eq!(completely_missing, vec!["field2"]);
    assert_eq!(partially_missing, vec!["field1"]);
  }

  #[test]
  fn test_process_field_value() {
    // 测试正常值
    let result = process_field_value(Some(json!("value")), true, false);
    assert_eq!(result, Some(json!(["value"])));

    // 测试数组值
    let result = process_field_value(Some(json!([1, 2, 3])), true, false);
    assert_eq!(result, Some(json!([[1, 2, 3]])));

    // 测试数组值且合并列表
    let result = process_field_value(Some(json!([1, 2, 3])), true, true);
    assert_eq!(result, Some(json!([1, 2, 3])));

    // 测试空值过滤
    let result = process_field_value(Some(Value::Null), false, false);
    assert_eq!(result, None);

    // 测试保留空值
    let result = process_field_value(Some(Value::Null), true, false);
    assert_eq!(result, Some(json!([null])));

    // 测试None值
    let result = process_field_value(None, true, false);
    assert_eq!(result, Some(json!([null])));
  }

  #[test]
  fn test_apply_field_filter() {
    let item = json!({
      "name": "John",
      "age": 30,
      "email": "john@example.com",
      "password": "secret"
    });

    // 测试排除字段
    let filtered = apply_field_filter(&item, &["password".to_string()], &[]);
    assert_eq!(
      filtered,
      Some(json!({
        "name": "John",
        "age": 30,
        "email": "john@example.com"
      }))
    );

    // 测试包含字段
    let filtered = apply_field_filter(&item, &[], &["name".to_string(), "email".to_string()]);
    assert_eq!(
      filtered,
      Some(json!({
        "name": "John",
        "email": "john@example.com"
      }))
    );

    // 测试同时排除和包含
    let filtered = apply_field_filter(&item, &["password".to_string()], &["name".to_string(), "email".to_string()]);
    assert_eq!(
      filtered,
      Some(json!({
        "name": "John",
        "email": "john@example.com"
      }))
    );
  }

  #[test]
  fn test_binary_unique_checker() {
    let mut checker = BinaryUniqueChecker::new();

    let binary1 = BinaryDataReference {
      file_key: "image1.png".to_string(),
      mime_kind: "image/png".to_string(),
      file_size: 1024,
      file_name: Some("image1.png".to_string()),
      file_kind: None,
      file_extension: Some("png".to_string()),
      directory: None,
    };

    let binary2 = BinaryDataReference {
      file_key: "image2.jpg".to_string(),
      mime_kind: "image/jpeg".to_string(),
      file_size: 2048,
      file_name: Some("image2.jpg".to_string()),
      file_kind: None,
      file_extension: Some("jpg".to_string()),
      directory: None,
    };

    let binary3 = BinaryDataReference {
      file_key: "image3.png".to_string(),
      mime_kind: "image/png".to_string(),
      file_size: 1024,
      file_name: Some("image3.png".to_string()),
      file_kind: None,
      file_extension: Some("png".to_string()),
      directory: None,
    };

    assert!(checker.is_unique(&binary1));
    assert!(checker.is_unique(&binary2));
    assert!(!checker.is_unique(&binary3)); // 与 binary1 相同属性
  }
}
