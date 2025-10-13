//! NoOp 节点工具函数
//!
//! 提供数据格式化、日志记录等实用功能。

use hetumind_core::workflow::ExecutionData;
use serde_json::Value;

/// 格式化数据摘要用于日志记录
///
/// 将执行数据格式化为适合日志记录的摘要字符串，
/// 避免在日志中输出过大的数据内容。
///
/// # 参数
/// - `data`: 要格式化的执行数据
///
/// # 返回值
/// 返回数据摘要字符串
pub fn format_data_summary(data: &ExecutionData) -> String {
  match data.json() {
    Value::Object(map) => {
      let field_count = map.len();
      if field_count == 0 {
        "空对象 {}".to_string()
      } else {
        let fields: Vec<String> = map.keys().take(3).cloned().collect();
        let more = if field_count > 3 { format!(" (+{})", field_count - 3) } else { String::new() };
        format!("对象 [{}]{}", fields.join(", "), more)
      }
    }
    Value::Array(arr) => {
      let len = arr.len();
      if len == 0 { "空数组 []".to_string() } else { format!("数组 [{} 项]", len) }
    }
    Value::String(s) => {
      if s.len() > 50 {
        format!("字符串 \"{}...\" ({})", &s[..47], s.len())
      } else {
        format!("字符串 \"{}\" ({})", s, s.len())
      }
    }
    Value::Number(n) => {
      format!("数字 {}", n)
    }
    Value::Bool(b) => {
      format!("布尔值 {}", b)
    }
    Value::Null => "null".to_string(),
  }
}

/// 计算数据的近似大小（字节）
///
/// 估算数据的内存占用大小，用于性能监控和日志记录。
/// 这不是精确的字节大小，而是用于相对比较的近似值。
///
/// # 参数
/// - `data`: 要计算大小的执行数据
///
/// # 返回值
/// 返回近似大小的字节数
pub fn estimate_data_size(data: &ExecutionData) -> usize {
  estimate_json_size(data.json())
}

/// 递归计算 JSON 值的近似大小
fn estimate_json_size(value: &Value) -> usize {
  match value {
    Value::Object(map) => {
      let mut size = 0;
      for (key, val) in map {
        size += key.len() + estimate_json_size(val);
      }
      size
    }
    Value::Array(arr) => arr.iter().map(estimate_json_size).sum(),
    Value::String(s) => s.len(),
    Value::Number(_) => {
      8 // 数字近似大小
    }
    Value::Bool(_) => {
      1 // 布尔值大小
    }
    Value::Null => {
      0 // null 值不占用空间
    }
  }
}

/// 验证数据完整性
///
/// 检查执行数据是否完整且有效。
/// 主要用于调试和验证目的。
///
/// # 参数
/// - `data`: 要验证的执行数据
///
/// # 返回值
/// 返回验证结果，包含是否有效和错误信息
pub fn validate_data_integrity(data: &ExecutionData) -> Result<(), String> {
  // 检查 JSON 数据是否有效
  let json_value = data.json();

  // 检查是否存在循环引用（serde_json 通常会处理这种情况）
  if let Err(e) = serde_json::to_string(json_value) {
    return Err(format!("JSON 序列化失败: {}", e));
  }

  // 检查数据大小是否合理（防止过大的数据）
  let estimated_size = estimate_data_size(data);
  if estimated_size > 100 * 1024 * 1024 {
    // 100MB 限制
    return Err(format!("数据过大: {} 字节", estimated_size));
  }

  Ok(())
}

/// 创建性能指标数据
///
/// 为性能监控创建包含执行指标的 JSON 对象。
///
/// # 参数
/// - `input_count`: 输入数据项数量
/// - `output_count`: 输出数据项数量
/// - `duration`: 执行耗时
/// - `data_size`: 数据大小（字节）
///
/// # 返回值
/// 返回包含性能指标的 JSON 值
pub fn create_performance_metrics(
  input_count: usize,
  output_count: usize,
  duration: std::time::Duration,
  data_size: usize,
) -> Value {
  serde_json::json!({
    "input_count": input_count,
    "output_count": output_count,
    "duration_ms": duration.as_millis(),
    "duration_us": duration.as_micros(),
    "data_size_bytes": data_size,
    "throughput_items_per_second": if duration.as_secs_f32() > 0.0 {
      output_count as f32 / duration.as_secs_f32()
    } else {
      0.0
    },
    "throughput_bytes_per_second": if duration.as_secs_f32() > 0.0 {
      data_size as f32 / duration.as_secs_f32()
    } else {
      0.0
    },
    "timestamp": chrono::Utc::now().to_rfc3339()
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use hetumind_core::workflow::ExecutionData;
  use serde_json::json;

  #[test]
  fn test_format_data_summary() {
    // 测试空对象
    let empty_obj = ExecutionData::new_json(json!({}), None);
    assert_eq!(format_data_summary(&empty_obj), "空对象 {}");

    // 测试对象
    let obj = ExecutionData::new_json(json!({"name": "John", "age": 30, "city": "NYC"}), None);
    let summary = format_data_summary(&obj);
    assert!(summary.contains("对象"));
    assert!(summary.contains("name"));

    // 测试数组
    let arr = ExecutionData::new_json(json!([1, 2, 3, 4, 5]), None);
    let summary = format_data_summary(&arr);
    assert_eq!(summary, "数组 [5 项]");

    // 测试字符串
    let short_str = ExecutionData::new_json(json!("Hello World"), None);
    let summary = format_data_summary(&short_str);
    assert!(summary.contains("字符串"));
    assert!(summary.contains("Hello World"));

    // 测试长字符串
    let long_str = ExecutionData::new_json(json!("A".repeat(100)), None);
    let summary = format_data_summary(&long_str);
    assert!(summary.contains("..."));
    assert!(summary.contains("100"));
  }

  #[test]
  fn test_estimate_data_size() {
    // 测试简单值
    let num = ExecutionData::new_json(json!(42), None);
    assert_eq!(estimate_data_size(&num), 8);

    let str_data = ExecutionData::new_json(json!("Hello"), None);
    assert_eq!(estimate_data_size(&str_data), 5);

    // 测试对象
    let obj = ExecutionData::new_json(json!({"a": 1, "b": 2}), None);
    let size = estimate_data_size(&obj);
    assert!(size > 10); // keys + values

    // 测试数组
    let arr = ExecutionData::new_json(json!([1, 2, 3]), None);
    let size = estimate_data_size(&arr);
    assert_eq!(size, 24); // 3 * 8 for numbers
  }

  #[test]
  fn test_validate_data_integrity() {
    // 测试有效数据
    let valid_data = ExecutionData::new_json(json!({"test": "data"}), None);
    assert!(validate_data_integrity(&valid_data).is_ok());

    // 测试 null 值
    let null_data = ExecutionData::new_json(json!(null), None);
    assert!(validate_data_integrity(&null_data).is_ok());
  }

  #[test]
  fn test_create_performance_metrics() {
    let metrics = create_performance_metrics(5, 5, std::time::Duration::from_millis(100), 1024);

    assert_eq!(metrics["input_count"], 5);
    assert_eq!(metrics["output_count"], 5);
    assert_eq!(metrics["duration_ms"], 100);
    assert_eq!(metrics["data_size_bytes"], 1024);
    assert!(metrics["throughput_items_per_second"].is_number());
    assert!(metrics["throughput_bytes_per_second"].is_number());
    assert!(metrics["timestamp"].is_string());
  }
}
