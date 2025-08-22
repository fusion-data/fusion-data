use log::{debug, error, warn};
use regex::Regex;
use ultimate_common::time::OffsetDateTime;

use hetumind_core::types::{DataType, JsonValue};
use hetumind_core::workflow::NodeExecutionError;

use super::{ComparisonOperation, ConditionConfig};

/// 转换为日期时间
pub fn to_datetime(value: &JsonValue) -> Option<OffsetDateTime> {
  match value {
    JsonValue::String(s) => {
      // 尝试解析 ISO 8601 格式的日期时间字符串
      // ultimate_common::time::OffsetDateTime 基于 time crate，支持 parse 方法
      s.parse().ok()
    }
    _ => None,
  }
}

/// 转换为数值
pub fn to_number(value: &JsonValue) -> Option<f64> {
  match value {
    JsonValue::Number(n) => n.as_f64(),
    JsonValue::String(s) => s.parse().ok(),
    _ => None,
  }
}

/// 评估单个条件
pub fn evaluate_single_condition(
  condition: &ConditionConfig,
  input_data: &JsonValue,
) -> Result<bool, NodeExecutionError> {
  // 解析 value1（可能是表达式）
  let value1 = resolve_value(&condition.left, input_data)?;
  let value2 = condition.right.as_ref().unwrap_or(&JsonValue::Null);

  debug!("比较: {:?} {:?} {:?}", value1, condition.op, value2);

  match condition.data_type {
    DataType::String => compare_strings(&value1, &condition.op, value2),
    DataType::Number => compare_numbers(&value1, &condition.op, value2),
    DataType::Boolean => compare_booleans(&value1, &condition.op, value2),
    DataType::DateTime { .. } => compare_datetimes(&value1, &condition.op, value2),
    // DataType::Date => self.compare_dates(&value1, &condition.op, value2),
    // DataType::Time => self.compare_times(&value1, &condition.op, value2),
    DataType::Object => {
      error!("不支持的数据类型: {:?}", condition.data_type);
      Ok(false)
    }
    _ => {
      // TODO
      Ok(false)
    }
  }
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

/// 解析值（支持简单的 JSON Path 表达式）
pub fn resolve_value(value: &JsonValue, input_data: &JsonValue) -> Result<JsonValue, NodeExecutionError> {
  if let Some(expr) = value.as_str()
    && let Some(path) = expr.strip_prefix("$.")
  {
    // 简单的 JSON Path 支持
    return Ok(get_nested_value(input_data, path).unwrap_or(JsonValue::Null));
  }
  Ok(value.clone())
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
