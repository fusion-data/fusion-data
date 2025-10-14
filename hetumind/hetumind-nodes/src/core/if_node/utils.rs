use fusion_common::time::OffsetDateTime;
use log::{debug, error, info, warn};
use regex::Regex;
use serde_json::json;
use serde_json_path::JsonPath;

use hetumind_core::types::{DataType, JsonValue};
use hetumind_core::workflow::NodeExecutionError;

use super::{
  ComparisonOperation, ConditionConfig, DefaultValueHandling, ErrorHandlingStrategy, IfNodeOptions, TypeValidationMode,
};

/// 转换为日期时间
pub fn to_datetime(value: &JsonValue) -> Option<OffsetDateTime> {
  match value {
    JsonValue::String(s) => {
      // 尝试解析为 OffsetDateTime
      if let Ok(dt) = s.parse::<OffsetDateTime>() {
        return Some(dt);
      }

      // TODO: 添加更多日期格式支持
      // 暂时简化实现，专注于 JSON Path 功能
      None
    }
    JsonValue::Number(n) => {
      // 如果是时间戳数字，需要转换为 OffsetDateTime
      // TODO: 实现时间戳转换
      None
    }
    _ => None,
  }
}

/// 转换为数值
pub fn to_number(value: &JsonValue) -> Option<f64> {
  match value {
    JsonValue::Number(n) => n.as_f64(),
    JsonValue::String(s) => {
      // 尝试解析为浮点数
      if let Ok(num) = s.parse::<f64>() {
        return Some(num);
      }

      // 尝试移除逗号等格式化字符后再解析
      let cleaned = s.replace(',', "").trim().to_string();
      if let Ok(num) = cleaned.parse::<f64>() {
        return Some(num);
      }

      // 尝试解析布尔值为 0/1
      match s.to_lowercase().as_str() {
        "true" | "1" => Some(1.0),
        "false" | "0" => Some(0.0),
        _ => None,
      }
    }
    JsonValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
    JsonValue::Null => Some(0.0),
    _ => None,
  }
}

/// 评估单个条件（带配置选项）
pub fn evaluate_single_condition_with_options(
  condition: &ConditionConfig,
  input_data: &JsonValue,
  options: &IfNodeOptions,
) -> Result<bool, NodeExecutionError> {
  info!("开始评估单个条件: {:?}", serde_json::to_string(condition).unwrap_or_default());
  info!("输入数据: {}", serde_json::to_string(input_data).unwrap_or_default());

  // 解析 value1（可能是表达式）
  let value1 = resolve_value_with_options(&condition.left, input_data, options)?;
  let value2 = condition
    .right
    .as_ref()
    .map(|v| resolve_value_with_options(v, input_data, options))
    .unwrap_or(Ok(get_default_value(&condition.data_type, options)))?;

  info!(
    "条件评估 - 左值: {:?}, 操作: {:?}, 右值: {:?}, 类型: {:?}, 忽略大小写: {}",
    value1, condition.op, value2, condition.data_type, options.ignore_case
  );

  let result = match condition.data_type {
    DataType::String => compare_strings_with_options(&value1, &condition.op, &value2, options),
    DataType::Number => compare_numbers_with_options(&value1, &condition.op, &value2, options),
    DataType::Boolean => compare_booleans_with_options(&value1, &condition.op, &value2, options),
    DataType::DateTime { .. } => compare_datetimes_with_options(&value1, &condition.op, &value2, options),
    // DataType::Date => compare_dates_with_options(&value1, &condition.op, value2, options),
    // DataType::Time => compare_times_with_options(&value1, &condition.op, value2, options),
    DataType::Object => {
      let error_msg = format!("不支持的数据类型: {:?}", condition.data_type);
      handle_evaluation_error(&error_msg, options)
    }
    _ => {
      // 尝试自动类型推断和比较
      if options.type_validation_mode == TypeValidationMode::Loose {
        auto_type_comparison(&value1, &condition.op, &value2, options)
      } else {
        let error_msg = format!("未知的数据类型: {:?}", condition.data_type);
        handle_evaluation_error(&error_msg, options)
      }
    }
  };

  if let Ok(result) = result {
    if options.debug_mode {
      debug!("条件评估结果: {}", result);
    }
    Ok(result)
  } else {
    Err(result.unwrap_err())
  }
}

/// 评估单个条件（保持向后兼容）
pub fn evaluate_single_condition(
  condition: &ConditionConfig,
  input_data: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  let options = IfNodeOptions::default();
  evaluate_single_condition_with_options(condition, input_data, &options)
}

/// 数值比较
pub fn compare_numbers(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  let num1 = to_number(value1).unwrap_or(0.0);
  let num2 = to_number(value2).unwrap_or(0.0);

  let result = match operation {
    ComparisonOperation::Eq => (num1 - num2).abs() < f64::EPSILON,
    ComparisonOperation::Nq => (num1 - num2).abs() >= f64::EPSILON,
    ComparisonOperation::Gt => num1 > num2,
    ComparisonOperation::Lt => num1 < num2,
    ComparisonOperation::Gte => num1 >= num2,
    ComparisonOperation::Lte => num1 <= num2,
    ComparisonOperation::Empty => value1.is_null(),
    ComparisonOperation::NotEmpty => !value1.is_null(),
    _ => {
      error!("数值类型不支持的比较操作: {:?}", operation);
      false
    }
  };

  Ok(result)
}

/// 布尔值比较
pub fn compare_booleans(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  let bool1 = value1.as_bool().unwrap_or(false);
  let bool2 = value2.as_bool().unwrap_or(false);

  let result = match operation {
    ComparisonOperation::Eq => bool1 == bool2,
    ComparisonOperation::Nq => bool1 != bool2,
    ComparisonOperation::Is => bool1,
    ComparisonOperation::Not => !bool1,
    ComparisonOperation::Empty => value1.is_null(),
    ComparisonOperation::NotEmpty => !value1.is_null(),
    _ => {
      error!("布尔值类型不支持的比较操作: {:?}", operation);
      false
    }
  };

  Ok(result)
}

/// 日期时间比较
pub fn compare_datetimes(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  let datetime1 = to_datetime(value1);
  let datetime2 = to_datetime(value2);

  if datetime1.is_none() || datetime2.is_none() {
    return Ok(false);
  }

  let dt1 = datetime1.unwrap();
  let dt2 = datetime2.unwrap();

  let result = match operation {
    ComparisonOperation::Eq => dt1 == dt2,
    ComparisonOperation::Nq => dt1 != dt2,
    ComparisonOperation::Gt => dt1 > dt2,
    ComparisonOperation::Lt => dt1 < dt2,
    ComparisonOperation::Gte => dt1 >= dt2,
    ComparisonOperation::Lte => dt1 <= dt2,
    ComparisonOperation::Empty => value1.is_null(),
    ComparisonOperation::NotEmpty => !value1.is_null(),
    _ => {
      error!("日期时间类型不支持的比较操作: {:?}", operation);
      false
    }
  };

  Ok(result)
}
/// 字符串比较
pub fn compare_strings(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  let str1 = value1.as_str().unwrap_or("");
  let str2 = value2.as_str().unwrap_or("");

  let result = match operation {
    ComparisonOperation::Eq => str1 == str2,
    ComparisonOperation::Nq => str1 != str2,
    ComparisonOperation::Contains => str1.contains(str2),
    ComparisonOperation::NotContains => !str1.contains(str2),
    ComparisonOperation::StartsWith => str1.starts_with(str2),
    ComparisonOperation::EndsWith => str1.ends_with(str2),
    ComparisonOperation::Regex => match Regex::new(str2) {
      Ok(re) => re.is_match(str1),
      Err(e) => {
        warn!("正则表达式错误: {}", e);
        false
      }
    },
    ComparisonOperation::Empty => str1.is_empty(),
    ComparisonOperation::NotEmpty => !str1.is_empty(),
    _ => {
      error!("字符串类型不支持的比较操作: {:?}", operation);
      false
    }
  };

  Ok(result)
}

/// 解析值（支持高级 JSON Path 表达式和 n8n 表达式）
pub fn resolve_value(value: &JsonValue, input_data: &JsonValue) -> Result<JsonValue, NodeExecutionError> {
  if let Some(expr) = value.as_str() {
    // 首先尝试解析 n8n 风格的表达式 {{ $json.field }}
    if expr.contains("{{") && expr.contains("}}") {
      return parse_expression_expression(expr, input_data);
    }

    // 支持完整的 JSON Path 表达式
    if let Ok(json_path) = JsonPath::parse(expr) {
      let results = json_path.query(input_data);

      if results.is_empty() {
        Ok(JsonValue::Null)
      } else if results.len() == 1 {
        Ok(results.first().unwrap().clone())
      } else {
        // 如果有多个匹配，返回数组
        let result_array: Vec<JsonValue> = results.into_iter().map(|v| v.clone()).collect();
        Ok(JsonValue::Array(result_array))
      }
    } else if let Some(path) = expr.strip_prefix("$.") {
      // 向后兼容：简单的 JSON Path 支持
      Ok(get_nested_value(input_data, path).unwrap_or(JsonValue::Null))
    } else {
      // 不是 JSON Path 表达式，返回原值
      Ok(value.clone())
    }
  } else {
    Ok(value.clone())
  }
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

/// 带配置选项的值解析
pub fn resolve_value_with_options(
  value: &JsonValue,
  input_data: &JsonValue,
  options: &IfNodeOptions,
) -> Result<JsonValue, NodeExecutionError> {
  let resolved = resolve_value(value, input_data)?;

  // 根据配置选项处理值
  if resolved.is_null() { Ok(apply_default_value_handling(&options.default_value_handling)) } else { Ok(resolved) }
}

/// 带配置选项的字符串比较
pub fn compare_strings_with_options(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
  options: &IfNodeOptions,
) -> Result<bool, NodeExecutionError> {
  let str1 = apply_case_handling(value1.as_str().unwrap_or(""), options.ignore_case);
  let str2 = apply_case_handling(value2.as_str().unwrap_or(""), options.ignore_case);

  let result = match operation {
    ComparisonOperation::Eq => str1 == str2,
    ComparisonOperation::Nq => str1 != str2,
    ComparisonOperation::Contains => str1.contains(&str2),
    ComparisonOperation::NotContains => !str1.contains(&str2),
    ComparisonOperation::StartsWith => str1.starts_with(&str2),
    ComparisonOperation::EndsWith => str1.ends_with(&str2),
    ComparisonOperation::Regex => match Regex::new(&str2) {
      Ok(re) => re.is_match(&str1),
      Err(e) => {
        let error_msg = format!("正则表达式错误: {}", e);
        return handle_comparison_error(&error_msg, options);
      }
    },
    ComparisonOperation::Empty => str1.is_empty(),
    ComparisonOperation::NotEmpty => !str1.is_empty(),
    _ => {
      let error_msg = format!("字符串类型不支持的比较操作: {:?}", operation);
      return handle_comparison_error(&error_msg, options);
    }
  };

  Ok(result)
}

/// 带配置选项的数值比较
pub fn compare_numbers_with_options(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
  options: &IfNodeOptions,
) -> Result<bool, NodeExecutionError> {
  let num1 = enhanced_to_number(value1, options);
  let num2 = enhanced_to_number(value2, options);

  let result = match operation {
    ComparisonOperation::Eq => (num1 - num2).abs() < f64::EPSILON,
    ComparisonOperation::Nq => (num1 - num2).abs() >= f64::EPSILON,
    ComparisonOperation::Gt => num1 > num2,
    ComparisonOperation::Lt => num1 < num2,
    ComparisonOperation::Gte => num1 >= num2,
    ComparisonOperation::Lte => num1 <= num2,
    ComparisonOperation::Empty => is_empty_numeric(value1, options),
    ComparisonOperation::NotEmpty => !is_empty_numeric(value1, options),
    _ => {
      let error_msg = format!("数值类型不支持的比较操作: {:?}", operation);
      return handle_comparison_error(&error_msg, options);
    }
  };

  Ok(result)
}

/// 带配置选项的布尔值比较
pub fn compare_booleans_with_options(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
  options: &IfNodeOptions,
) -> Result<bool, NodeExecutionError> {
  let bool1 = enhanced_to_boolean(value1, options);
  let bool2 = enhanced_to_boolean(value2, options);

  let result = match operation {
    ComparisonOperation::Eq => bool1 == bool2,
    ComparisonOperation::Nq => bool1 != bool2,
    ComparisonOperation::Is => bool1,
    ComparisonOperation::Not => !bool1,
    ComparisonOperation::Empty => is_empty_boolean(value1, options),
    ComparisonOperation::NotEmpty => !is_empty_boolean(value1, options),
    _ => {
      let error_msg = format!("布尔值类型不支持的比较操作: {:?}", operation);
      return handle_comparison_error(&error_msg, options);
    }
  };

  Ok(result)
}

/// 带配置选项的日期时间比较
pub fn compare_datetimes_with_options(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
  options: &IfNodeOptions,
) -> Result<bool, NodeExecutionError> {
  let datetime1 = enhanced_to_datetime(value1, options);
  let datetime2 = enhanced_to_datetime(value2, options);

  if datetime1.is_none() || datetime2.is_none() {
    return Ok(false);
  }

  let dt1 = datetime1.unwrap();
  let dt2 = datetime2.unwrap();

  let result = match operation {
    ComparisonOperation::Eq => dt1 == dt2,
    ComparisonOperation::Nq => dt1 != dt2,
    ComparisonOperation::Gt => dt1 > dt2,
    ComparisonOperation::Lt => dt1 < dt2,
    ComparisonOperation::Gte => dt1 >= dt2,
    ComparisonOperation::Lte => dt1 <= dt2,
    ComparisonOperation::Empty => is_empty_datetime(value1, options),
    ComparisonOperation::NotEmpty => !is_empty_datetime(value1, options),
    _ => {
      let error_msg = format!("日期时间类型不支持的比较操作: {:?}", operation);
      return handle_comparison_error(&error_msg, options);
    }
  };

  Ok(result)
}

/// 自动类型推断和比较
pub fn auto_type_comparison(
  value1: &JsonValue,
  operation: &ComparisonOperation,
  value2: &JsonValue,
  options: &IfNodeOptions,
) -> Result<bool, NodeExecutionError> {
  // 尝试数值比较
  if let (Some(num1), Some(num2)) =
    (enhanced_to_number_option(value1, options), enhanced_to_number_option(value2, options))
  {
    if options.debug_mode {
      debug!("自动类型推断: 数值比较 {} {} {}", num1, format_operation(operation), num2);
    }
    return compare_numbers_with_options(&json!(num1), operation, &json!(num2), options);
  }

  // 尝试布尔值比较
  if let (Some(bool1), Some(bool2)) =
    (enhanced_to_boolean_maybe(value1, options), enhanced_to_boolean_maybe(value2, options))
  {
    if options.debug_mode {
      debug!("自动类型推断: 布尔比较 {} {} {}", bool1, format_operation(operation), bool2);
    }
    return compare_booleans_with_options(&json!(bool1), operation, &json!(bool2), options);
  }

  // 默认字符串比较
  if options.debug_mode {
    debug!("自动类型推断: 字符串比较 {} {} {}", value1, format_operation(operation), value2);
  }
  compare_strings_with_options(value1, operation, value2, options)
}

/// 错误处理函数
fn handle_evaluation_error(error_msg: &str, options: &IfNodeOptions) -> Result<bool, NodeExecutionError> {
  match options.error_handling_strategy {
    ErrorHandlingStrategy::StopExecution => {
      error!("停止执行: {}", error_msg);
      Err(NodeExecutionError::ExecutionFailed {
        node_name: "IfNode".to_string().into(),
        message: Some(error_msg.to_string()),
      })
    }
    ErrorHandlingStrategy::SkipItem => {
      warn!("跳过当前项: {}", error_msg);
      Ok(false)
    }
    ErrorHandlingStrategy::GoToDefaultBranch => {
      info!("走默认分支: {}", error_msg);
      Ok(false)
    }
    ErrorHandlingStrategy::LogAndContinue => {
      warn!("记录错误但继续: {}", error_msg);
      Ok(false)
    }
  }
}

/// 比较错误处理
fn handle_comparison_error(error_msg: &str, options: &IfNodeOptions) -> Result<bool, NodeExecutionError> {
  handle_evaluation_error(error_msg, options)
}

/// 应用大小写处理
fn apply_case_handling(s: &str, ignore_case: bool) -> String {
  if ignore_case { s.to_lowercase() } else { s.to_string() }
}

/// 获取默认值
fn get_default_value(data_type: &DataType, options: &IfNodeOptions) -> JsonValue {
  match &options.default_value_handling {
    DefaultValueHandling::UseNull => JsonValue::Null,
    DefaultValueHandling::UseEmptyString => JsonValue::String("".to_string()),
    DefaultValueHandling::UseZero => JsonValue::Number(serde_json::Number::from(0)),
    DefaultValueHandling::UseFalse => JsonValue::Bool(false),
    DefaultValueHandling::Custom(value) => value.clone(),
  }
}

/// 应用默认值处理策略
fn apply_default_value_handling(strategy: &DefaultValueHandling) -> JsonValue {
  match strategy {
    DefaultValueHandling::UseNull => JsonValue::Null,
    DefaultValueHandling::UseEmptyString => JsonValue::String("".to_string()),
    DefaultValueHandling::UseZero => JsonValue::Number(serde_json::Number::from(0)),
    DefaultValueHandling::UseFalse => JsonValue::Bool(false),
    DefaultValueHandling::Custom(value) => value.clone(),
  }
}

/// 增强的数值转换
fn enhanced_to_number(value: &JsonValue, options: &IfNodeOptions) -> f64 {
  match options.type_validation_mode {
    TypeValidationMode::Strict => value.as_f64().unwrap_or_else(|| {
      if options.continue_on_error {
        0.0
      } else {
        panic!("严格的数值转换失败: {:?}", value);
      }
    }),
    TypeValidationMode::Loose | TypeValidationMode::Smart => to_number(value).unwrap_or(0.0),
  }
}

/// 增强的数值转换（返回Option）
fn enhanced_to_number_option(value: &JsonValue, options: &IfNodeOptions) -> Option<f64> {
  match options.type_validation_mode {
    TypeValidationMode::Strict => value.as_f64(),
    TypeValidationMode::Loose | TypeValidationMode::Smart => to_number(value),
  }
}

/// 增强的布尔值转换
fn enhanced_to_boolean(value: &JsonValue, options: &IfNodeOptions) -> bool {
  match options.type_validation_mode {
    TypeValidationMode::Strict => value.as_bool().unwrap_or_else(|| {
      if options.continue_on_error {
        false
      } else {
        panic!("严格的布尔值转换失败: {:?}", value);
      }
    }),
    TypeValidationMode::Loose | TypeValidationMode::Smart => match value {
      JsonValue::Bool(b) => *b,
      JsonValue::String(s) => match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => false,
      },
      JsonValue::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
      JsonValue::Null => false,
      _ => false,
    },
  }
}

/// 可能的布尔值转换（不强制转换）
fn enhanced_to_boolean_maybe(value: &JsonValue, _options: &IfNodeOptions) -> Option<bool> {
  match value {
    JsonValue::Bool(b) => Some(*b),
    JsonValue::String(s) => match s.to_lowercase().as_str() {
      "true" | "1" | "yes" | "on" => Some(true),
      "false" | "0" | "no" | "off" => Some(false),
      _ => None,
    },
    _ => None,
  }
}

/// 增强的日期时间转换
fn enhanced_to_datetime(value: &JsonValue, options: &IfNodeOptions) -> Option<OffsetDateTime> {
  match options.type_validation_mode {
    TypeValidationMode::Strict => to_datetime(value),
    TypeValidationMode::Loose | TypeValidationMode::Smart => {
      // 更宽松的日期时间解析
      to_datetime(value)
    }
  }
}

/// 检查数值是否为空
fn is_empty_numeric(value: &JsonValue, _options: &IfNodeOptions) -> bool {
  match value {
    JsonValue::Null => true,
    JsonValue::String(s) => s.trim().is_empty(),
    _ => false,
  }
}

/// 检查布尔值是否为空
fn is_empty_boolean(value: &JsonValue, _options: &IfNodeOptions) -> bool {
  matches!(value, JsonValue::Null)
}

/// 检查日期时间是否为空
fn is_empty_datetime(value: &JsonValue, _options: &IfNodeOptions) -> bool {
  matches!(value, JsonValue::Null)
}

/// 格式化操作符用于调试
fn format_operation(op: &ComparisonOperation) -> String {
  match op {
    ComparisonOperation::Eq => "==".to_string(),
    ComparisonOperation::Nq => "!=".to_string(),
    ComparisonOperation::Gt => ">".to_string(),
    ComparisonOperation::Lt => "<".to_string(),
    ComparisonOperation::Gte => ">=".to_string(),
    ComparisonOperation::Lte => "<=".to_string(),
    ComparisonOperation::Contains => "contains".to_string(),
    ComparisonOperation::NotContains => "not contains".to_string(),
    ComparisonOperation::StartsWith => "starts with".to_string(),
    ComparisonOperation::EndsWith => "ends with".to_string(),
    ComparisonOperation::Regex => "matches".to_string(),
    ComparisonOperation::Empty => "is empty".to_string(),
    ComparisonOperation::NotEmpty => "is not empty".to_string(),
    ComparisonOperation::Is => "is true".to_string(),
    ComparisonOperation::Not => "is false".to_string(),
  }
}

/// 条件表达式解析器（支持类似 n8n 的表达式语法）
pub fn parse_expression_expression(expression: &str, input_data: &JsonValue) -> Result<JsonValue, NodeExecutionError> {
  // 简化的表达式解析实现
  // 支持: {{ $json.field }}, {{ $now }}, {{ $random }} 等

  if let Some(captures) = parse_simple_expression(expression) {
    if let Some(expr_match) = captures.get(1) {
      let expr = expr_match.as_str().trim();

      if expr.starts_with("$json.") {
        // JSON Path 表达式
        let path = &expr[6..]; // 移除 "$json."
        // 直接使用 get_nested_value 而不是 resolve_value，避免双重解析
        match get_nested_value(input_data, path) {
          Some(value) => Ok(value),
          None => {
            warn!("JSON 路径 '{}' 未找到，返回 null", path);
            Ok(JsonValue::Null)
          }
        }
      } else if expr == "$now" {
        // 当前时间
        let now = fusion_common::time::now();
        Ok(json!(now.to_string()))
      } else if expr == "$random" {
        // 随机数
        use rand::Rng;
        let mut rng = rand::rng();
        let random_value: f64 = rng.random();
        Ok(json!(random_value))
      } else if expr.starts_with("$env.") {
        // 环境变量
        let env_var = &expr[5..];
        if let Ok(value) = std::env::var(env_var) { Ok(json!(value)) } else { Ok(JsonValue::Null) }
      } else {
        // 其他表达式类型（暂时返回原值）
        warn!("不支持的表达式类型: {}", expr);
        Ok(JsonValue::String(expression.to_string()))
      }
    } else {
      Ok(JsonValue::String(expression.to_string()))
    }
  } else {
    // 不是表达式，返回原值
    Ok(JsonValue::String(expression.to_string()))
  }
}

/// 解析简单表达式 {{ expression }}
fn parse_simple_expression(expression: &str) -> Option<regex::Captures> {
  let re = Regex::new(r"\{\{\s*([^}]+)\s*\}\}").ok()?;
  re.captures(expression)
}

/// 应用表达式到值
pub fn apply_expressions_to_value(value: &JsonValue, input_data: &JsonValue) -> Result<JsonValue, NodeExecutionError> {
  match value {
    JsonValue::String(s) => {
      // 检查是否包含表达式
      if s.contains("{{") && s.contains("}}") { parse_expression_expression(s, input_data) } else { Ok(value.clone()) }
    }
    _ => Ok(value.clone()),
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_string_comparison() {
    // 测试字符串相等
    let result = compare_strings(&json!("hello"), &ComparisonOperation::Eq, &json!("hello")).unwrap();
    assert!(result);

    // 测试字符串包含
    let result = compare_strings(&json!("hello world"), &ComparisonOperation::Contains, &json!("world")).unwrap();
    assert!(result);

    // 测试正则表达式
    let result =
      compare_strings(&json!("test@example.com"), &ComparisonOperation::Regex, &json!(r".*@.*\.com")).unwrap();
    assert!(result);
  }

  #[test]
  fn test_number_comparison() {
    // 测试数值大于
    let result = compare_numbers(&json!(10), &ComparisonOperation::Gt, &json!(5)).unwrap();
    assert!(result);

    // 测试数值等于
    let result = compare_numbers(&json!(5.5), &ComparisonOperation::Eq, &json!(5.5)).unwrap();
    assert!(result);
  }

  #[test]
  fn test_boolean_comparison() {
    // 测试布尔值比较
    let result = compare_booleans(&json!(true), &ComparisonOperation::Is, &json!(true)).unwrap();
    assert!(result);

    let result = compare_booleans(&json!(false), &ComparisonOperation::Not, &json!(false)).unwrap();
    assert!(result);
  }
}
