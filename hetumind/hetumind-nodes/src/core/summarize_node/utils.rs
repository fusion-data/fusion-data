//! Summarize 节点工具函数
//!
//! 提供数据聚合、分组和格式化的核心工具函数。

use hetumind_core::workflow::{ExecutionData, NodeExecutionError};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

use super::{
  AggregateField, AggregateOperation, DataType, ErrorHandlingStrategy, GroupByConfig, GroupSortOrder, OutputFormat,
  SerializationStyle, SummarizeConfig,
};

/// 聚合结果
#[derive(Debug, Clone)]
pub struct AggregateResult {
  /// 聚合值
  pub value: Value,
  /// 数据类型
  pub data_type: DataType,
  /// 计数（用于统计）
  pub count: usize,
  /// 元数据
  pub metadata: Option<HashMap<String, Value>>,
}

/// 分组聚合结果
#[derive(Debug, Clone)]
pub struct GroupAggregateResult {
  /// 分组键
  pub group_key: Value,
  /// 聚合结果
  pub aggregates: HashMap<String, AggregateResult>,
  /// 原始数据项（可选）
  pub original_items: Option<Vec<Value>>,
}

/// 从配置聚合数据
///
/// 这是 Summarize 节点的核心聚合逻辑，负责将输入数据转换为聚合结果。
///
/// # 参数
/// - `input_items`: 输入数据项
/// - `config`: 聚合配置
/// - `node_name`: 节点名称（用于错误消息）
///
/// # 返回
/// 返回聚合后的数据项列表。
///
/// # 聚合策略
/// 1. 如果有分组配置，先按字段分组再聚合
/// 2. 如果没有分组，直接对所有数据进行聚合
/// 3. 根据错误处理策略处理无效数据
/// 4. 支持多种数据类型和聚合操作
pub fn aggregate_data(
  input_items: &[ExecutionData],
  config: &SummarizeConfig,
  node_name: &str,
) -> Result<Vec<Value>, NodeExecutionError> {
  if input_items.is_empty() {
    log::warn!("节点 {} 没有输入数据，返回空结果", node_name);
    return Ok(vec![]);
  }

  log::debug!("开始聚合数据: 输入项={}, 分组配置={}", input_items.len(), config.group_by.is_some());

  let result = if let Some(ref group_config) = config.group_by {
    // 分组聚合
    aggregate_by_groups(input_items, config, group_config)?
  } else {
    // 全局聚合
    let aggregates = calculate_aggregates(input_items, &config.aggregate_fields, &config.get_error_handling())?;
    let mut result = serde_json::Map::new();

    // 应用序列化风格到字段名
    for (field_name, aggregate_result) in aggregates {
      let formatted_name = convert_field_name(&field_name, &config.serialization_style);
      result.insert(formatted_name, aggregate_result.value);
    }

    // 添加元数据
    if config.should_include_metadata() {
      result.insert("_metadata".to_string(), create_metadata(input_items, config)?);
    }

    vec![Value::Object(result)]
  };

  log::debug!("聚合完成: 输出项={}", result.len());
  Ok(result)
}

/// 按组聚合数据
fn aggregate_by_groups(
  input_items: &[ExecutionData],
  config: &SummarizeConfig,
  group_config: &GroupByConfig,
) -> Result<Vec<Value>, NodeExecutionError> {
  let mut groups: HashMap<Value, Vec<ExecutionData>> = HashMap::new();

  // 分组数据
  for item in input_items {
    let group_value = extract_field_value(item.json(), &group_config.group_field).unwrap_or(Value::Null);

    groups.entry(group_value).or_insert_with(Vec::new).push(item.clone());
  }

  log::debug!("分组完成: 组数={}", groups.len());

  let mut results = Vec::new();

  for (group_key, group_items) in groups {
    let aggregates = calculate_aggregates(&group_items, &config.aggregate_fields, &config.get_error_handling())?;

    let mut result = serde_json::Map::new();

    // 添加分组键
    let group_output_name = convert_field_name(&group_config.get_group_output_name(), &config.serialization_style);
    result.insert(group_output_name, group_key.clone());

    // 添加聚合结果
    for (field_name, aggregate_result) in aggregates {
      let formatted_name = convert_field_name(&field_name, &config.serialization_style);
      result.insert(formatted_name, aggregate_result.value);
    }

    // 添加原始数据（如果配置了）
    if group_config.should_keep_original_data() {
      let original_data: Vec<Value> = group_items.iter().map(|item| item.json().clone()).collect();
      result.insert("original_data".to_string(), Value::Array(original_data));
    }

    // 添加元数据
    if config.should_include_metadata() {
      result.insert("_metadata".to_string(), create_group_metadata(&group_key, &group_items, config)?);
    }

    results.push(Value::Object(result));
  }

  // 排序结果
  sort_results(&mut results, group_config)?;

  Ok(results)
}

/// 计算聚合值
fn calculate_aggregates(
  items: &[ExecutionData],
  aggregate_fields: &[AggregateField],
  error_handling: &ErrorHandlingStrategy,
) -> Result<HashMap<String, AggregateResult>, NodeExecutionError> {
  let mut results = HashMap::new();

  for field in aggregate_fields {
    let field_result = calculate_single_aggregate(items, field, error_handling)?;
    results.insert(field.output_field.clone(), field_result);
  }

  Ok(results)
}

/// 计算单个字段的聚合值
fn calculate_single_aggregate(
  items: &[ExecutionData],
  field: &AggregateField,
  error_handling: &ErrorHandlingStrategy,
) -> Result<AggregateResult, NodeExecutionError> {
  let mut values = Vec::new();
  let mut valid_count = 0;
  let mut null_count = 0;

  // 提取字段值
  for item in items {
    match extract_field_value(item.json(), &field.source_field) {
      Some(value) => {
        if value.is_null() && field.should_ignore_empty() {
          null_count += 1;
          continue;
        }

        match convert_value_to_type(&value, &field.data_type) {
          Ok(converted_value) => {
            values.push(converted_value);
            valid_count += 1;
          }
          Err(e) => handle_conversion_error(e, error_handling, &field.source_field)?,
        }
      }
      None => {
        if !field.should_ignore_empty() {
          handle_conversion_error(
            format!("Field '{}' not found", field.source_field),
            error_handling,
            &field.source_field,
          )?;
        }
        null_count += 1;
      }
    }
  }

  log::debug!("字段 {}: 有效值={}, 空值={}", field.output_field, valid_count, null_count);

  // 计算聚合结果
  let aggregate_value = match field.operation {
    AggregateOperation::Count => Value::Number(serde_json::Number::from(valid_count)),
    AggregateOperation::CountEmpty => Value::Number(serde_json::Number::from(null_count)),
    AggregateOperation::CountNotEmpty => Value::Number(serde_json::Number::from(valid_count)),
    AggregateOperation::Sum => calculate_sum(&values)?,
    AggregateOperation::Avg => calculate_avg(&values)?,
    AggregateOperation::Min => calculate_min(&values)?,
    AggregateOperation::Max => calculate_max(&values)?,
    AggregateOperation::Median => calculate_median(&values)?,
    AggregateOperation::StdDev => calculate_stddev(&values)?,
    AggregateOperation::Variance => calculate_variance(&values)?,
    AggregateOperation::Concat => calculate_concat(&values, "")?,
    AggregateOperation::Join => calculate_concat(&values, &field.get_separator())?,
    AggregateOperation::CountUnique => Value::Number(serde_json::Number::from(count_unique_values(&values))),
    AggregateOperation::First => values.first().cloned().unwrap_or(Value::Null),
    AggregateOperation::Last => values.last().cloned().unwrap_or(Value::Null),
  };

  let data_type = infer_data_type(&aggregate_value);

  let mut metadata = HashMap::new();
  metadata.insert("count".to_string(), Value::Number(serde_json::Number::from(valid_count)));
  metadata.insert("null_count".to_string(), Value::Number(serde_json::Number::from(null_count)));

  Ok(AggregateResult { value: aggregate_value, data_type, count: valid_count, metadata: Some(metadata) })
}

/// 提取字段值（支持 JSON 路径）
fn extract_field_value(data: &Value, path: &str) -> Option<Value> {
  if path == "{{ $json }}" || path == "$json" || path == "" {
    return Some(data.clone());
  }

  // 简单的 JSON 路径解析（支持点分隔的路径）
  if path.starts_with("{{") && path.ends_with("}}") {
    let clean_path = path.trim_start_matches("{{").trim_end_matches("}}").trim();
    return extract_nested_value(data, clean_path);
  }

  // 直接字段名
  extract_nested_value(data, path)
}

/// 提取嵌套值
fn extract_nested_value(data: &Value, path: &str) -> Option<Value> {
  let parts: Vec<&str> = path.split('.').collect();
  let mut current = data;

  for part in parts {
    match current {
      Value::Object(map) => match map.get(part) {
        Some(value) => current = value,
        None => return None,
      },
      Value::Array(arr) => {
        if let Ok(index) = part.parse::<usize>() {
          if index < arr.len() {
            current = &arr[index];
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

  Some(current.clone())
}

/// 转换值到指定类型
fn convert_value_to_type(value: &Value, target_type: &Option<DataType>) -> Result<Value, String> {
  let target_type = target_type.as_ref().unwrap_or(&DataType::String);

  match (target_type, value) {
    (DataType::String, _) => Ok(Value::String(format!("{}", value))),
    (DataType::Number, Value::String(s)) => s
      .parse::<f64>()
      .map(|n| Value::Number(serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0))))
      .map_err(|_| format!("Cannot convert '{}' to number", s)),
    (DataType::Number, Value::Number(n)) => Ok(Value::Number(n.clone())),
    (DataType::Boolean, Value::String(s)) => match s.to_lowercase().as_str() {
      "true" | "1" | "yes" | "on" => Ok(Value::Bool(true)),
      "false" | "0" | "no" | "off" => Ok(Value::Bool(false)),
      _ => Err(format!("Cannot convert '{}' to boolean", s)),
    },
    (DataType::Boolean, Value::Bool(b)) => Ok(Value::Bool(*b)),
    _ => Ok(value.clone()),
  }
}

/// 处理转换错误
fn handle_conversion_error(
  error: String,
  error_handling: &ErrorHandlingStrategy,
  field_name: &str,
) -> Result<(), NodeExecutionError> {
  match error_handling {
    ErrorHandlingStrategy::SkipError => {
      log::warn!("字段 {} 转换错误: {} (跳过)", field_name, error);
      Ok(())
    }
    ErrorHandlingStrategy::UseDefault => {
      log::warn!("字段 {} 转换错误: {} (使用默认值)", field_name, error);
      Ok(())
    }
    ErrorHandlingStrategy::StopExecution => Err(NodeExecutionError::DataProcessingError {
      message: format!("Field conversion error for '{}': {}", field_name, error),
    }),
    ErrorHandlingStrategy::LogAndContinue => {
      log::warn!("字段 {} 转换错误: {} (记录但继续)", field_name, error);
      Ok(())
    }
  }
}

// 数学计算函数
fn calculate_sum(values: &[Value]) -> Result<Value, NodeExecutionError> {
  let mut sum = 0.0;
  for value in values {
    match value {
      Value::Number(n) => sum += n.as_f64().unwrap_or(0.0),
      Value::String(s) => {
        sum += s.parse::<f64>().map_err(|_| NodeExecutionError::DataProcessingError {
          message: format!("Cannot convert '{}' to number for sum", s),
        })?;
      }
      _ => {}
    }
  }
  Ok(Value::Number(serde_json::Number::from_f64(sum).unwrap_or(serde_json::Number::from(0))))
}

fn calculate_avg(values: &[Value]) -> Result<Value, NodeExecutionError> {
  if values.is_empty() {
    return Ok(Value::Number(serde_json::Number::from(0)));
  }

  let sum = calculate_sum(values)?;
  if let Value::Number(n) = sum {
    let avg = n.as_f64().unwrap_or(0.0) / values.len() as f64;
    Ok(Value::Number(serde_json::Number::from_f64(avg).unwrap_or(serde_json::Number::from(0))))
  } else {
    Err(NodeExecutionError::DataProcessingError { message: "Failed to calculate average".to_string() })
  }
}

fn calculate_min(values: &[Value]) -> Result<Value, NodeExecutionError> {
  if values.is_empty() {
    return Ok(Value::Null);
  }

  let mut min_value = None;
  for value in values {
    let numeric_value = match value {
      Value::Number(n) => n.as_f64().unwrap_or(f64::MAX),
      Value::String(s) => s.parse::<f64>().unwrap_or(f64::MAX),
      _ => continue,
    };

    if min_value.is_none() || numeric_value < min_value.unwrap() {
      min_value = Some(numeric_value);
    }
  }

  Ok(
    min_value
      .map(|v| Value::Number(serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from(0))))
      .unwrap_or(Value::Null),
  )
}

fn calculate_max(values: &[Value]) -> Result<Value, NodeExecutionError> {
  if values.is_empty() {
    return Ok(Value::Null);
  }

  let mut max_value = None;
  for value in values {
    let numeric_value = match value {
      Value::Number(n) => n.as_f64().unwrap_or(f64::MIN),
      Value::String(s) => s.parse::<f64>().unwrap_or(f64::MIN),
      _ => continue,
    };

    if max_value.is_none() || numeric_value > max_value.unwrap() {
      max_value = Some(numeric_value);
    }
  }

  Ok(
    max_value
      .map(|v| Value::Number(serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from(0))))
      .unwrap_or(Value::Null),
  )
}

fn calculate_median(values: &[Value]) -> Result<Value, NodeExecutionError> {
  if values.is_empty() {
    return Ok(Value::Null);
  }

  let mut numeric_values: Vec<f64> = values
    .iter()
    .filter_map(|value| match value {
      Value::Number(n) => n.as_f64(),
      Value::String(s) => s.parse::<f64>().ok(),
      _ => None,
    })
    .collect();

  if numeric_values.is_empty() {
    return Ok(Value::Null);
  }

  numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

  let median = if numeric_values.len() % 2 == 0 {
    let mid = numeric_values.len() / 2;
    (numeric_values[mid - 1] + numeric_values[mid]) / 2.0
  } else {
    numeric_values[numeric_values.len() / 2]
  };

  Ok(Value::Number(serde_json::Number::from_f64(median).unwrap_or(serde_json::Number::from(0))))
}

fn calculate_stddev(values: &[Value]) -> Result<Value, NodeExecutionError> {
  let variance = calculate_variance(values)?;
  if let Value::Number(n) = variance {
    let stddev = n.as_f64().unwrap_or(0.0).sqrt();
    Ok(Value::Number(serde_json::Number::from_f64(stddev).unwrap_or(serde_json::Number::from(0))))
  } else {
    Ok(Value::Number(serde_json::Number::from(0)))
  }
}

fn calculate_variance(values: &[Value]) -> Result<Value, NodeExecutionError> {
  if values.is_empty() {
    return Ok(Value::Number(serde_json::Number::from(0)));
  }

  let avg = calculate_avg(values)?;
  if let Value::Number(avg_n) = avg {
    let avg_value = avg_n.as_f64().unwrap_or(0.0);
    let mut sum_of_squares = 0.0;

    for value in values {
      let numeric_value = match value {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
      };
      let diff = numeric_value - avg_value;
      sum_of_squares += diff * diff;
    }

    let variance = sum_of_squares / values.len() as f64;
    Ok(Value::Number(serde_json::Number::from_f64(variance).unwrap_or(serde_json::Number::from(0))))
  } else {
    Ok(Value::Number(serde_json::Number::from(0)))
  }
}

fn calculate_concat(values: &[Value], separator: &str) -> Result<Value, NodeExecutionError> {
  let strings: Vec<String> = values
    .iter()
    .map(|value| match value {
      Value::String(s) => s.clone(),
      _ => value.to_string(),
    })
    .collect();

  Ok(Value::String(strings.join(separator)))
}

fn count_unique_values(values: &[Value]) -> usize {
  let unique_values: HashSet<&Value> = values.iter().collect();
  unique_values.len()
}

/// 推断数据类型
fn infer_data_type(value: &Value) -> DataType {
  match value {
    Value::String(_) => DataType::String,
    Value::Number(_) => DataType::Number,
    Value::Bool(_) => DataType::Boolean,
    Value::Array(_) => DataType::Array,
    Value::Object(_) => DataType::Object,
    Value::Null => DataType::String, // 将 null 视为字符串
  }
}

/// 转换字段名格式
pub fn convert_field_name(name: &str, style: &SerializationStyle) -> String {
  match style {
    SerializationStyle::SnakeCase => to_snake_case(name),
    SerializationStyle::CamelCase => to_camel_case(name),
    SerializationStyle::PascalCase => to_pascal_case(name),
    SerializationStyle::KebabCase => to_kebab_case(name),
  }
}

/// 转换为 snake_case
fn to_snake_case(name: &str) -> String {
  let mut result = String::new();
  let mut prev_char_was_upper = false;

  for (i, c) in name.chars().enumerate() {
    if c.is_uppercase() {
      if i > 0 && !prev_char_was_upper && result.chars().last().map_or(false, |last| last != '_' && last != '-') {
        result.push('_');
      }
      result.push(c.to_lowercase().next().unwrap_or(c));
      prev_char_was_upper = true;
    } else if c == '-' || c == ' ' {
      if result.chars().last().map_or(true, |last| last != '_') {
        result.push('_');
      }
      prev_char_was_upper = false;
    } else {
      result.push(c);
      prev_char_was_upper = false;
    }
  }

  result
}

/// 转换为 camelCase
fn to_camel_case(name: &str) -> String {
  let parts: Vec<String> = name
    .split(&['_', '-', ' '])
    .filter(|s| !s.is_empty())
    .enumerate()
    .map(|(i, part)| {
      if i == 0 {
        part.to_lowercase()
      } else {
        let mut chars = part.chars();
        match chars.next() {
          None => String::new(),
          Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        }
      }
    })
    .collect();

  parts.join("")
}

/// 转换为 PascalCase
fn to_pascal_case(name: &str) -> String {
  let parts: Vec<String> = name
    .split(&['_', '-', ' '])
    .filter(|s| !s.is_empty())
    .map(|part| {
      let mut chars = part.chars();
      match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
      }
    })
    .collect();

  if parts.is_empty() { name.to_string() } else { parts.join("") }
}

/// 转换为 kebab-case
fn to_kebab_case(name: &str) -> String {
  to_snake_case(name).replace('_', "-")
}

/// 排序结果
fn sort_results(results: &mut Vec<Value>, group_config: &GroupByConfig) -> Result<(), NodeExecutionError> {
  if let Some(sort_order) = &group_config.sort_by {
    let group_output_name = &group_config.group_output_name;

    results.sort_by(|a, b| {
      let key_a = extract_nested_value(a, group_output_name).unwrap_or(Value::Null);
      let key_b = extract_nested_value(b, group_output_name).unwrap_or(Value::Null);

      match sort_order {
        GroupSortOrder::GroupAsc => compare_values(&key_a, &key_b),
        GroupSortOrder::GroupDesc => compare_values(&key_b, &key_a),
        GroupSortOrder::CountAsc => {
          let count_a = extract_count_from_result(a).unwrap_or(0);
          let count_b = extract_count_from_result(b).unwrap_or(0);
          count_a.cmp(&count_b)
        }
        GroupSortOrder::CountDesc => {
          let count_a = extract_count_from_result(a).unwrap_or(0);
          let count_b = extract_count_from_result(b).unwrap_or(0);
          count_b.cmp(&count_a)
        }
        GroupSortOrder::None => std::cmp::Ordering::Equal,
      }
    });
  }

  Ok(())
}

/// 比较两个值
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
  match (a, b) {
    (Value::String(sa), Value::String(sb)) => sa.cmp(sb),
    (Value::Number(na), Value::Number(nb)) => {
      let fa = na.as_f64().unwrap_or(0.0);
      let fb = nb.as_f64().unwrap_or(0.0);
      fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
    }
    (Value::Bool(ba), Value::Bool(bb)) => ba.cmp(bb),
    _ => a.to_string().cmp(&b.to_string()),
  }
}

/// 从结果中提取计数
fn extract_count_from_result(result: &Value) -> Option<usize> {
  if let Value::Object(map) = result {
    // 尝试从聚合字段中找到计数
    for (_key, value) in map {
      if let Value::Number(n) = value {
        return Some(n.as_u64()? as usize);
      }
    }
  }
  None
}

/// 创建元数据
fn create_metadata(items: &[ExecutionData], config: &SummarizeConfig) -> Result<Value, NodeExecutionError> {
  let mut metadata = serde_json::Map::new();

  metadata.insert("total_items".to_string(), Value::Number(serde_json::Number::from(items.len())));
  metadata.insert("aggregated_at".to_string(), Value::String(chrono::Utc::now().to_rfc3339()));
  metadata
    .insert("aggregate_fields".to_string(), Value::Number(serde_json::Number::from(config.aggregate_fields.len())));

  if config.group_by.is_some() {
    metadata.insert("grouped".to_string(), Value::Bool(true));
  }

  Ok(Value::Object(metadata))
}

/// 创建分组元数据
fn create_group_metadata(
  group_key: &Value,
  group_items: &[ExecutionData],
  _config: &SummarizeConfig,
) -> Result<Value, NodeExecutionError> {
  let mut metadata = serde_json::Map::new();

  metadata.insert("group_key".to_string(), group_key.clone());
  metadata.insert("group_size".to_string(), Value::Number(serde_json::Number::from(group_items.len())));
  metadata.insert("aggregated_at".to_string(), Value::String(chrono::Utc::now().to_rfc3339()));

  Ok(Value::Object(metadata))
}

/// 格式化输出
pub fn format_output(aggregated_data: &[Value], config: &SummarizeConfig) -> Result<Vec<Value>, NodeExecutionError> {
  match config.output_format {
    OutputFormat::Json => Ok(aggregated_data.to_vec()),
    OutputFormat::KeyValueArray => convert_to_key_value_array(aggregated_data, config),
    OutputFormat::TableFormat => convert_to_table_format(aggregated_data, config),
  }
}

/// 转换为键值对数组
fn convert_to_key_value_array(data: &[Value], config: &SummarizeConfig) -> Result<Vec<Value>, NodeExecutionError> {
  let mut result = Vec::new();

  for item in data {
    if let Value::Object(map) = item {
      let mut key_value_pairs = Vec::new();

      for (key, value) in map {
        let pair = json!({
          "key": convert_field_name(key, &config.serialization_style),
          "value": value
        });
        key_value_pairs.push(pair);
      }

      result.push(Value::Array(key_value_pairs));
    }
  }

  Ok(result)
}

/// 转换为表格格式
fn convert_to_table_format(data: &[Value], config: &SummarizeConfig) -> Result<Vec<Value>, NodeExecutionError> {
  let mut result = Vec::new();

  for item in data {
    if let Value::Object(map) = item {
      let mut table_row = serde_json::Map::new();

      for (key, value) in map {
        let formatted_key = convert_field_name(key, &config.serialization_style);
        table_row.insert(formatted_key, value.clone());
      }

      result.push(Value::Object(table_row));
    }
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_field_name_conversion() {
    assert_eq!(convert_field_name("user_name", &SerializationStyle::SnakeCase), "user_name");
    assert_eq!(convert_field_name("userName", &SerializationStyle::SnakeCase), "user_name");
    assert_eq!(convert_field_name("User-Name", &SerializationStyle::SnakeCase), "user_name");

    assert_eq!(convert_field_name("user_name", &SerializationStyle::CamelCase), "userName");
    assert_eq!(convert_field_name("user_name", &SerializationStyle::PascalCase), "UserName");
    assert_eq!(convert_field_name("user_name", &SerializationStyle::KebabCase), "user-name");
  }

  #[test]
  fn test_extract_field_value() {
    let data = json!({
      "user": {
        "name": "John",
        "age": 30
      },
      "tags": ["tag1", "tag2"]
    });

    assert_eq!(extract_field_value(&data, "user.name"), Some(json!("John")));
    assert_eq!(extract_field_value(&data, "user.age"), Some(json!(30)));
    assert_eq!(extract_field_value(&data, "tags.0"), Some(json!("tag1")));
    assert_eq!(extract_field_value(&data, "nonexistent"), None);
  }

  #[test]
  fn test_calculate_sum() {
    let values = vec![json!(1), json!(2), json!(3), json!("4")];
    let result = calculate_sum(&values).unwrap();
    assert_eq!(result, json!(10.0));

    let empty_values: Vec<Value> = vec![];
    let result = calculate_sum(&empty_values).unwrap();
    assert_eq!(result, json!(0.0));
  }

  #[test]
  fn test_calculate_avg() {
    let values = vec![json!(1), json!(2), json!(3), json!(4)];
    let result = calculate_avg(&values).unwrap();
    assert_eq!(result, json!(2.5));

    let empty_values: Vec<Value> = vec![];
    let result = calculate_avg(&empty_values).unwrap();
    assert_eq!(result, json!(0));
  }

  #[test]
  fn test_calculate_concat() {
    let values = vec![json!("hello"), json!("world"), json!("test")];
    let result = calculate_concat(&values, ", ").unwrap();
    assert_eq!(result, json!("hello, world, test"));

    let result_empty = calculate_concat(&values, "").unwrap();
    assert_eq!(result_empty, json!("helloworldtest"));
  }

  #[test]
  fn test_count_unique_values() {
    let values = vec![json!("a"), json!("b"), json!("a"), json!("c"), json!("b")];
    let count = count_unique_values(&values);
    assert_eq!(count, 3);

    let values = vec![json!(1), json!(2), json!(1), json!(3)];
    let count = count_unique_values(&values);
    assert_eq!(count, 3);
  }

  #[test]
  fn test_infer_data_type() {
    assert_eq!(infer_data_type(&json!("hello")), DataType::String);
    assert_eq!(infer_data_type(&json!(42)), DataType::Number);
    assert_eq!(infer_data_type(&json!(true)), DataType::Boolean);
    assert_eq!(infer_data_type(&json!([])), DataType::Array);
    assert_eq!(infer_data_type(&json!({})), DataType::Object);
    assert_eq!(infer_data_type(&json!(null)), DataType::String);
  }

  #[test]
  fn test_convert_value_to_type() {
    // 字符串到数字
    let result = convert_value_to_type(&json!("42"), &Some(DataType::Number));
    assert_eq!(result.unwrap(), json!(42.0));

    // 数字到字符串
    let result = convert_value_to_type(&json!(42), &Some(DataType::String));
    assert_eq!(result.unwrap(), json!("42"));

    // 字符串到布尔值
    let result = convert_value_to_type(&json!("true"), &Some(DataType::Boolean));
    assert_eq!(result.unwrap(), json!(true));

    let result = convert_value_to_type(&json!("false"), &Some(DataType::Boolean));
    assert_eq!(result.unwrap(), json!(false));
  }
}
