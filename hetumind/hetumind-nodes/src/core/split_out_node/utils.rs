//! Split Out 节点工具函数
//!
//! 提供数据处理、字段访问、数据标准化等实用功能。

use hetumind_core::types::JsonValue;
use serde_json::Value;
use std::collections::HashMap;

use super::{IncludeStrategy, FieldToSplit, SplitOutConfig};

/// 准备字段数组，将字符串或数组转换为标准化的字段列表
pub fn prepare_fields_array(fields: &str, _field_name: &str) -> Vec<String> {
  if fields.trim().is_empty() {
    return vec![];
  }

  fields
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect()
}

/// 提取字段数据，支持点记号路径
pub fn extract_field_data(data: &JsonValue, field_path: &str, disable_dot_notation: bool) -> Option<JsonValue> {
  if disable_dot_notation {
    // 直接属性访问
    data.get(field_path).cloned()
  } else {
    // 支持点记号路径
    get_nested_value(data, field_path)
  }
}

/// 递归获取嵌套值
fn get_nested_value(data: &JsonValue, path: &str) -> Option<JsonValue> {
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

/// 标准化数据为可拆分格式
pub fn normalize_data_to_split(data: JsonValue) -> Vec<JsonValue> {
  match data {
    Value::Null => {
      vec![] // null 转换为空数组
    }
    Value::Array(arr) => {
      // 数组直接返回
      arr
    }
    Value::Object(obj) => {
      // 对象转换为值数组
      obj.into_values().collect()
    }
    _ => {
      // 基本类型（字符串、数字、布尔值）包装为单元素数组
      vec![data]
    }
  }
}

/// 缺失字段跟踪器
pub struct MissingFieldsTracker {
  not_found_fields: HashMap<String, Vec<bool>>,
}

impl MissingFieldsTracker {
  /// 创建新的跟踪器
  pub fn new() -> Self {
    Self {
      not_found_fields: HashMap::new(),
    }
  }

  /// 记录字段是否存在
  pub fn record_field_existence(&mut self, field: &str, exists: bool) {
    self.not_found_fields
      .entry(field.to_string())
      .or_insert_with(Vec::new)
      .push(exists);
  }

  /// 获取完全缺失的字段列表
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

  /// 获取所有字段的存在情况
  pub fn get_field_existence_summary(&self) -> HashMap<String, FieldExistenceSummary> {
    let mut summary = HashMap::new();

    for (field, existence) in &self.not_found_fields {
      let total = existence.len();
      let found = existence.iter().filter(|&&exists| exists).count();
      let missing = total - found;

      let existence_type = if found == 0 {
        FieldExistenceType::NeverFound
      } else if missing == 0 {
        FieldExistenceType::AlwaysFound
      } else {
        FieldExistenceType::PartiallyFound
      };

      summary.insert(field.clone(), FieldExistenceSummary {
        total_items: total,
        found_items: found,
        missing_items: missing,
        existence_type,
      });
    }

    summary
  }
}

/// 字段存在性摘要
#[derive(Debug, Clone)]
pub struct FieldExistenceSummary {
  /// 总项目数
  pub total_items: usize,
  /// 找到的项目数
  pub found_items: usize,
  /// 缺失的项目数
  pub missing_items: usize,
  /// 存在性类型
  pub existence_type: FieldExistenceType,
}

/// 字段存在性类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldExistenceType {
  /// 从未找到
  NeverFound,
  /// 总是找到
  AlwaysFound,
  /// 部分找到
  PartiallyFound,
}

/// 数据类型分析器
pub struct DataTypeAnalyzer;

impl DataTypeAnalyzer {
  /// 分析数据类型
  pub fn analyze_data_type(data: &JsonValue) -> DataTypeInfo {
    match data {
      Value::Null => DataTypeInfo {
        base_type: BaseType::Null,
        is_array: false,
        is_object: false,
        depth: 0,
        size_estimate: 0,
      },
      Value::Bool(_) => DataTypeInfo {
        base_type: BaseType::Boolean,
        is_array: false,
        is_object: false,
        depth: 0,
        size_estimate: 1,
      },
      Value::Number(_) => DataTypeInfo {
        base_type: BaseType::Number,
        is_array: false,
        is_object: false,
        depth: 0,
        size_estimate: 8,
      },
      Value::String(s) => DataTypeInfo {
        base_type: BaseType::String,
        is_array: false,
        is_object: false,
        depth: 0,
        size_estimate: s.len(),
      },
      Value::Array(arr) => {
        let depth = Self::calculate_depth(data);
        DataTypeInfo {
          base_type: BaseType::Array,
          is_array: true,
          is_object: false,
          depth,
          size_estimate: arr.len(),
        }
      }
      Value::Object(obj) => {
        let depth = Self::calculate_depth(data);
        DataTypeInfo {
          base_type: BaseType::Object,
          is_array: false,
          is_object: true,
          depth,
          size_estimate: obj.len(),
        }
      }
    }
  }

  /// 计算数据结构的深度
  fn calculate_depth(data: &JsonValue) -> usize {
    match data {
      Value::Array(arr) => {
        if arr.is_empty() {
          1
        } else {
          // 检查是否包含嵌套的数组或对象
          let has_nested = arr.iter().any(|v| matches!(v, Value::Array(_) | Value::Object(_)));
          if has_nested {
            let max_child_depth = arr.iter().map(|v| Self::calculate_depth(v)).max().unwrap_or(1);
            1 + max_child_depth
          } else {
            1 // 简单数组，深度为1
          }
        }
      }
      Value::Object(obj) => {
        if obj.is_empty() {
          1
        } else {
          // 检查是否包含嵌套的数组或对象
          let has_nested = obj.values().any(|v| matches!(v, Value::Array(_) | Value::Object(_)));
          if has_nested {
            let max_child_depth = obj.values().map(|v| Self::calculate_depth(v)).max().unwrap_or(1);
            1 + max_child_depth
          } else {
            1 // 简单对象，深度为1
          }
        }
      }
      _ => 1,
    }
  }

  /// 估算拆分后的项目数量
  pub fn estimate_split_output_count(data: &JsonValue) -> usize {
    match data {
      Value::Array(arr) => arr.len(),
      Value::Object(obj) => obj.len(),
      Value::Null => 0,
      _ => 1, // 基本类型包装为单元素数组
    }
  }

  /// 分析拆分操作的内存影响
  pub fn analyze_memory_impact(data: &JsonValue, field_count: usize) -> MemoryImpactAnalysis {
    let original_size = Self::estimate_memory_size(data);
    let output_count = Self::estimate_split_output_count(data);

    // 估算每个输出项的大小（考虑字段复制）
    let avg_output_size = if field_count > 0 {
      // 简化估算：假设每个输出项包含原始数据的一部分
      original_size / std::cmp::max(1, output_count) * (field_count + 1) / 2
    } else {
      original_size / std::cmp::max(1, output_count)
    };

    let estimated_output_size = avg_output_size * output_count;
    let memory_overhead = estimated_output_size.saturating_sub(original_size);

    MemoryImpactAnalysis {
      original_size,
      estimated_output_size,
      memory_overhead,
      estimated_output_items: output_count,
      memory_efficiency_ratio: if original_size > 0 {
        original_size as f64 / estimated_output_size as f64
      } else {
        1.0
      },
    }
  }

  /// 估算内存大小（字节）
  fn estimate_memory_size(data: &JsonValue) -> usize {
    match data {
      Value::Null => 0,
      Value::Bool(_) => 1,
      Value::Number(_) => 8,
      Value::String(s) => s.len(),
      Value::Array(arr) => {
        arr.iter().map(|v| Self::estimate_memory_size(v)).sum()
      }
      Value::Object(obj) => {
        obj.iter().map(|(k, v)| k.len() + Self::estimate_memory_size(v)).sum()
      }
    }
  }
}

/// 数据类型信息
#[derive(Debug, Clone)]
pub struct DataTypeInfo {
  /// 基础类型
  pub base_type: BaseType,
  /// 是否为数组
  pub is_array: bool,
  /// 是否为对象
  pub is_object: bool,
  /// 数据深度
  pub depth: usize,
  /// 大小估算
  pub size_estimate: usize,
}

/// 基础数据类型
#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
  Null,
  Boolean,
  Number,
  String,
  Array,
  Object,
}

/// 内存影响分析
#[derive(Debug, Clone)]
pub struct MemoryImpactAnalysis {
  /// 原始数据大小
  pub original_size: usize,
  /// 估算输出大小
  pub estimated_output_size: usize,
  /// 内存开销
  pub memory_overhead: usize,
  /// 估算输出项目数
  pub estimated_output_items: usize,
  /// 内存效率比率
  pub memory_efficiency_ratio: f64,
}

/// 拆分操作验证器
pub struct SplitOperationValidator;

impl SplitOperationValidator {
  /// 验证拆分操作的可行性
  pub fn validate_split_operation(
    data: &JsonValue,
    field_path: &str,
    config: &super::SplitOutConfig,
  ) -> ValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // 检查字段是否存在
    let field_data = extract_field_data(data, field_path, config.disable_dot_notation);
    if field_data.is_none() {
      issues.push(ValidationIssue {
        severity: ValidationSeverity::Error,
        code: "FIELD_NOT_FOUND".to_string(),
        message: format!("Field '{}' not found in data", field_path),
        suggestion: Some("Check the field path and ensure it exists in the input data".to_string()),
      });
      return ValidationResult { issues, warnings };
    }

    let field_data = field_data.unwrap();
    let type_info = DataTypeAnalyzer::analyze_data_type(&field_data);

    // 检查数据类型是否适合拆分
    match type_info.base_type {
      BaseType::Array => {
        if type_info.size_estimate == 0 {
          warnings.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "EMPTY_ARRAY".to_string(),
            message: "Array is empty, no items will be generated".to_string(),
            suggestion: Some("Consider adding a filter before the split operation".to_string()),
          });
        }
      }
      BaseType::Object => {
        if type_info.size_estimate == 0 {
          warnings.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "EMPTY_OBJECT".to_string(),
            message: "Object is empty, no items will be generated".to_string(),
            suggestion: Some("Consider adding a filter before the split operation".to_string()),
          });
        }
      }
      BaseType::String => {
        warnings.push(ValidationIssue {
          severity: ValidationSeverity::Info,
          code: "PRIMITIVE_TYPE".to_string(),
          message: "Splitting a primitive value will create a single item".to_string(),
          suggestion: Some("This is usually fine, but ensure it's the intended behavior".to_string()),
        });
      }
      BaseType::Null => {
        issues.push(ValidationIssue {
          severity: ValidationSeverity::Warning,
          code: "NULL_VALUE".to_string(),
          message: "Field contains null value, no items will be generated".to_string(),
          suggestion: Some("Consider using a default value or filtering out null values".to_string()),
        });
      }
      _ => {
        // Number, Boolean - 这些是正常的
      }
    }

    // 内存影响分析
    let memory_analysis = DataTypeAnalyzer::analyze_memory_impact(&field_data, config.fields_to_split.len());
    if memory_analysis.memory_efficiency_ratio < 0.1 {
      warnings.push(ValidationIssue {
        severity: ValidationSeverity::Warning,
        code: "HIGH_MEMORY_USAGE".to_string(),
        message: format!("This operation may use {}x more memory", (1.0 / memory_analysis.memory_efficiency_ratio) as u32),
        suggestion: Some("Consider processing data in batches or reducing the number of fields to include".to_string()),
      });
    }

    // 配置验证
    if config.include_strategy == IncludeStrategy::SelectedOtherFields && config.fields_to_include.is_empty() {
      issues.push(ValidationIssue {
        severity: ValidationSeverity::Error,
        code: "EMPTY_SELECTED_FIELDS".to_string(),
        message: "Selected other fields strategy requires at least one field to include".to_string(),
        suggestion: Some("Add fields to include or change the include strategy".to_string()),
      });
    }

    ValidationResult { issues, warnings }
  }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
  /// 验证问题
  pub issues: Vec<ValidationIssue>,
  /// 验证警告
  pub warnings: Vec<ValidationIssue>,
}

/// 验证问题
#[derive(Debug, Clone)]
pub struct ValidationIssue {
  /// 严重程度
  pub severity: ValidationSeverity,
  /// 问题代码
  pub code: String,
  /// 问题消息
  pub message: String,
  /// 建议解决方案
  pub suggestion: Option<String>,
}

/// 验证严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
  Error,
  Warning,
  Info,
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
  fn test_extract_field_data() {
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
    assert_eq!(
      extract_field_data(&data, "name", true),
      Some(json!("John"))
    );
    assert_eq!(
      extract_field_data(&data, "nonexistent", true),
      None
    );

    // 点记号访问
    assert_eq!(
      extract_field_data(&data, "profile.age", false),
      Some(json!(30))
    );
    assert_eq!(
      extract_field_data(&data, "profile.address.city", false),
      Some(json!("New York"))
    );

    // 数组索引访问
    assert_eq!(
      extract_field_data(&data, "tags.0", false),
      Some(json!("tag1"))
    );
  }

  #[test]
  fn test_normalize_data_to_split() {
    // 测试数组
    let arr = json!([1, 2, 3]);
    let result = normalize_data_to_split(arr);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], json!(1));
    assert_eq!(result[1], json!(2));
    assert_eq!(result[2], json!(3));

    // 测试对象
    let obj = json!({"a": 1, "b": 2});
    let result = normalize_data_to_split(obj);
    assert_eq!(result.len(), 2);
    assert!(result.contains(&json!(1)));
    assert!(result.contains(&json!(2)));

    // 测试基本类型
    let primitive = json!("hello");
    let result = normalize_data_to_split(primitive);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], json!("hello"));

    // 测试 null
    let null_val = json!(null);
    let result = normalize_data_to_split(null_val);
    assert_eq!(result.len(), 0);
  }

  #[test]
  fn test_missing_fields_tracker() {
    let mut tracker = MissingFieldsTracker::new();

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
  fn test_data_type_analyzer() {
    // 测试数组分析
    let arr = json!([1, 2, 3]);
    let info = DataTypeAnalyzer::analyze_data_type(&arr);
    assert_eq!(info.base_type, BaseType::Array);
    assert!(info.is_array);
    assert!(!info.is_object);
    assert_eq!(info.depth, 1);
    assert_eq!(info.size_estimate, 3);

    // 测试嵌套对象分析
    let nested = json!({"a": {"b": {"c": 1}}});
    let info = DataTypeAnalyzer::analyze_data_type(&nested);
    assert_eq!(info.base_type, BaseType::Object);
    assert!(!info.is_array);
    assert!(info.is_object);
    assert_eq!(info.depth, 3);
    assert_eq!(info.size_estimate, 1);

    // 测试估算输出数量
    assert_eq!(DataTypeAnalyzer::estimate_split_output_count(&json!([1, 2, 3])), 3);
    assert_eq!(DataTypeAnalyzer::estimate_split_output_count(&json!({"a": 1, "b": 2})), 2);
    assert_eq!(DataTypeAnalyzer::estimate_split_output_count(&json!("hello")), 1);
    assert_eq!(DataTypeAnalyzer::estimate_split_output_count(&json!(null)), 0);
  }

  #[test]
  fn test_memory_impact_analysis() {
    let data = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
    let analysis = DataTypeAnalyzer::analyze_memory_impact(&data, 1);

    assert_eq!(analysis.estimated_output_items, 2);
    assert!(analysis.original_size > 0);
    // The analysis should show some memory usage change
    assert!(analysis.memory_efficiency_ratio > 0.0);
  }

  #[test]
  fn test_split_operation_validator() {
    let data = json!({
      "items": [1, 2, 3],
      "other": "value"
    });

    let config = SplitOutConfig {
      fields_to_split: vec![FieldToSplit {
        field_to_split: "items".to_string(),
        destination_field: None,
      }],
      include_strategy: IncludeStrategy::AllOtherFields,
      fields_to_include: vec![],
      disable_dot_notation: false,
      include_binary: false,
    };

    let result = SplitOperationValidator::validate_split_operation(&data, "items", &config);
    assert!(result.issues.is_empty());
  }
}