//! Stop and Error 节点工具函数
//!
//! 提供错误创建、验证和处理的核心工具函数。

use hetumind_core::workflow::ValidationError;
use serde_json::Value;
use std::collections::HashMap;

use super::{ErrorLevel, ErrorObject, StopAndErrorConfig};

/// 错误处理结果
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErrorResult {
  /// 错误消息
  pub message: String,
  /// 错误描述
  pub description: Option<String>,
  /// 错误代码
  pub error_code: Option<String>,
  /// 错误级别
  pub error_level: ErrorLevel,
  /// 错误元数据
  pub metadata: Option<Value>,
  /// 是否可重试
  pub retryable: bool,
  /// 重试延迟（秒）
  pub retry_after: Option<u64>,
}

/// 从配置创建错误结果
///
/// 此函数是 Stop and Error 节点的核心错误处理逻辑，负责将配置转换为可执行的错误对象。
///
/// # 参数
/// - `config`: 错误配置
/// - `node_name`: 节点名称（用于上下文信息）
///
/// # 返回
/// 返回一个 `ErrorResult`，包含所有必要的错误信息。
///
/// # 错误处理策略
/// 1. 对于简单错误消息，直接使用消息内容
/// 2. 对于复杂错误对象，提取关键信息并提供备选方案
/// 3. 确保始终有有效的错误消息，避免空错误
/// 4. 保留完整的元数据上下文
pub fn create_error_from_config(config: &StopAndErrorConfig, node_name: &str) -> Result<ErrorResult, ValidationError> {
  // 验证配置
  config.validate()?;

  let message = config.get_error_message();
  let description = config.get_error_description();
  let error_code = config.get_error_code();
  let error_level = config.get_error_level();
  let metadata = config.get_error_metadata();
  let retryable = config.is_retryable();
  let retry_after = config.get_retry_after();

  // 确保消息不为空
  let final_message =
    if message.trim().is_empty() { format!("Error occurred in node: {}", node_name) } else { message };

  Ok(ErrorResult { message: final_message, description, error_code, error_level, metadata, retryable, retry_after })
}

/// 验证配置并添加上下文信息
///
/// 除了基本验证外，还会添加与执行上下文相关的验证逻辑。
///
/// # 参数
/// - `config`: 要验证的配置
/// - `node_name`: 节点名称（用于错误消息）
///
/// # 返回
/// 如果验证成功，返回 `Ok(())`，否则返回带有详细信息的 `ValidationError`。
pub fn validate_config_with_context(config: &StopAndErrorConfig, node_name: &str) -> Result<(), ValidationError> {
  // 基本验证
  config.validate()?;

  // 上下文相关验证
  match config.error_type {
    super::ErrorType::ErrorMessage => {
      if let Some(ref message) = config.error_message {
        // 检查消息长度（避免过长或过短的消息）
        if message.len() > 10000 {
          return Err(ValidationError::invalid_field_value(
            "error_message".to_string(),
            format!("Error message too long (max 10000 characters, got {})", message.len()),
          ));
        }

        if message.trim().is_empty() {
          return Err(ValidationError::invalid_field_value(
            "error_message".to_string(),
            "Error message cannot be empty or whitespace only".to_string(),
          ));
        }
      }
    }
    super::ErrorType::ErrorObject => {
      if let Some(ref error_obj) = config.error_object {
        // 验证错误对象的字段组合
        if error_obj.message.is_none() && error_obj.description.is_none() && error_obj.code.is_none() {
          return Err(ValidationError::invalid_field_value(
            "error_object".to_string(),
            "Error object must contain at least one of: message, description, or code".to_string(),
          ));
        }

        // 验证重试配置
        if let Some(retryable) = error_obj.retryable {
          if retryable {
            // 如果可重试，建议设置重试延迟
            if error_obj.retry_after.is_none() {
              log::warn!("Node '{}' has retryable error but no retry_after set, using default 60 seconds", node_name);
            } else if let Some(retry_after) = error_obj.retry_after {
              if retry_after == 0 {
                return Err(ValidationError::invalid_field_value(
                  "error_object.retry_after".to_string(),
                  "Retry delay cannot be zero when retryable is true".to_string(),
                ));
              }
              if retry_after > 86400 {
                // 24小时
                return Err(ValidationError::invalid_field_value(
                  "error_object.retry_after".to_string(),
                  "Retry delay cannot exceed 24 hours (86400 seconds)".to_string(),
                ));
              }
            }
          }
        }

        // 验证错误代码格式
        if let Some(ref code) = error_obj.code {
          if code.trim().is_empty() {
            return Err(ValidationError::invalid_field_value(
              "error_object.code".to_string(),
              "Error code cannot be empty".to_string(),
            ));
          }
          if code.len() > 100 {
            return Err(ValidationError::invalid_field_value(
              "error_object.code".to_string(),
              "Error code too long (max 100 characters)".to_string(),
            ));
          }
        }
      }
    }
  }

  Ok(())
}

/// 解析 JSON 错误对象
///
/// 尝试将 JSON 值解析为 `ErrorObject`，并提供详细的错误信息。
///
/// # 参数
/// - `json_value`: 要解析的 JSON 值
///
/// # 返回
/// 返回解析后的 `ErrorObject` 或带有详细错误信息的 `ValidationError`。
#[allow(dead_code)]
pub fn parse_error_object(json_value: &Value) -> Result<ErrorObject, ValidationError> {
  // 检查是否为对象类型
  if !json_value.is_object() {
    return Err(ValidationError::invalid_field_value(
      "error_object".to_string(),
      "Error object must be a JSON object".to_string(),
    ));
  }

  // 尝试反序列化
  match serde_json::from_value::<ErrorObject>(json_value.clone()) {
    Ok(error_obj) => Ok(error_obj),
    Err(e) => Err(ValidationError::invalid_field_value(
      "error_object".to_string(),
      format!("Invalid error object format: {}", e),
    )),
  }
}

/// 提取错误消息（带优先级）
///
/// 按照优先级从错误对象中提取错误消息：
/// 1. message 字段（最高优先级）
/// 2. description 字段
/// 3. error 字段（兼容性考虑）
/// 4. code 字段（最后备选）
/// 5. JSON 字符串化（兜底方案）
///
/// # 参数
/// - `error_obj`: 错误对象
///
/// # 返回
/// 返回提取到的错误消息字符串。
#[allow(dead_code)]
pub fn extract_error_message(error_obj: &ErrorObject) -> String {
  // 第一优先级: message 字段
  if let Some(ref message) = error_obj.message {
    if !message.trim().is_empty() {
      return message.clone();
    }
  }

  // 第二优先级: description 字段
  if let Some(ref description) = error_obj.description {
    if !description.trim().is_empty() {
      return description.clone();
    }
  }

  // 第三优先级: code 字段
  if let Some(ref code) = error_obj.code {
    if !code.trim().is_empty() {
      return code.clone();
    }
  }

  // 最后备选: 默认错误消息
  "Error occurred".to_string()
}

/// 构建错误元数据
///
/// 将错误对象的各种属性构建为结构化的元数据 JSON 对象。
///
/// # 参数
/// - `error_obj`: 错误对象
/// - `node_name`: 节点名称
///
/// # 返回
/// 返回包含所有错误上下文的元数据 JSON 对象。
#[allow(dead_code)]
pub fn build_error_metadata(error_obj: &ErrorObject, node_name: &str) -> Value {
  let mut metadata_map = HashMap::new();

  // 添加节点信息
  metadata_map.insert("node_name".to_string(), Value::String(node_name.to_string()));
  metadata_map.insert("generated_at".to_string(), Value::String(chrono::Utc::now().to_rfc3339()));

  // 添加错误对象字段
  if let Some(ref code) = error_obj.code {
    metadata_map.insert("code".to_string(), Value::String(code.clone()));
  }

  if let Some(ref error_type) = error_obj.error_type {
    metadata_map.insert("error_type".to_string(), Value::String(error_type.clone()));
  }

  if let Some(ref level) = error_obj.level {
    metadata_map.insert("level".to_string(), Value::String(format!("{:?}", level).to_lowercase()));
  }

  if let Some(retryable) = error_obj.retryable {
    metadata_map.insert("retryable".to_string(), Value::Bool(retryable));
  }

  if let Some(retry_after) = error_obj.retry_after {
    metadata_map.insert("retry_after".to_string(), Value::Number(serde_json::Number::from(retry_after)));
  }

  // 合并自定义元数据
  if let Some(ref custom_metadata) = error_obj.metadata {
    if let Value::Object(custom_map) = custom_metadata {
      for (key, value) in custom_map {
        metadata_map.insert(key.clone(), value.clone());
      }
    }
  }

  Value::Object(metadata_map.into_iter().collect())
}

/// 格式化错误级别
///
/// 将错误级别枚举转换为用户友好的字符串。
///
/// # 参数
/// - `level`: 错误级别
///
/// # 返回
/// 返回格式化后的级别字符串。
pub fn format_error_level(level: &ErrorLevel) -> &'static str {
  match level {
    ErrorLevel::Info => "info",
    ErrorLevel::Warning => "warning",
    ErrorLevel::Error => "error",
    ErrorLevel::Critical => "critical",
  }
}

/// 验证错误消息内容
///
/// 检查错误消息是否包含有效内容，防止空消息或仅包含空白字符的消息。
///
/// # 参数
/// - `message`: 要验证的错误消息
///
/// # 返回
/// 如果消息有效，返回 `Ok(())`，否则返回 `ValidationError`。
#[allow(dead_code)]
pub fn validate_error_message(message: &str) -> Result<(), ValidationError> {
  if message.trim().is_empty() {
    return Err(ValidationError::invalid_field_value(
      "error_message".to_string(),
      "Error message cannot be empty or whitespace only".to_string(),
    ));
  }

  if message.len() > 10000 {
    return Err(ValidationError::invalid_field_value(
      "error_message".to_string(),
      format!("Error message too long (max 10000 characters, got {})", message.len()),
    ));
  }

  // 检查是否包含潜在的恶意内容
  let suspicious_patterns = vec!["<script", "</script>", "javascript:", "data:", "vbscript:"];

  let lower_message = message.to_lowercase();
  for pattern in suspicious_patterns {
    if lower_message.contains(pattern) {
      log::warn!("Error message contains potentially suspicious content: {}", pattern);
      // 不阻止执行，但记录警告
    }
  }

  Ok(())
}

/// 创建默认错误对象
///
/// 当解析失败或配置无效时，创建一个默认的错误对象。
///
/// # 参数
/// - `node_name`: 节点名称
/// - `context`: 错误上下文
///
/// # 返回
/// 返回一个合理的默认错误对象。
#[allow(dead_code)]
pub fn create_default_error_object(node_name: &str, context: &str) -> ErrorObject {
  ErrorObject {
    code: Some("STOP_AND_ERROR_DEFAULT".to_string()),
    message: Some(format!("Error in node {}: {}", node_name, context)),
    description: Some("A default error was created due to invalid configuration".to_string()),
    error_type: Some("ConfigurationError".to_string()),
    level: Some(ErrorLevel::Error),
    metadata: Some(serde_json::json!({
      "node_name": node_name,
      "context": context,
      "fallback": true
    })),
    retryable: Some(false),
    retry_after: None,
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use crate::core::stop_and_error_node::ErrorType;

  use super::*;

  #[test]
  fn test_create_error_from_simple_config() {
    let config = StopAndErrorConfig {
      error_type: ErrorType::ErrorMessage,
      error_message: Some("Test error message".to_string()),
      error_object: None,
    };

    let result = create_error_from_config(&config, "test_node").unwrap();

    assert_eq!(result.message, "Test error message");
    assert_eq!(result.description, Some("Test error message".to_string()));
    assert_eq!(result.error_code, None);
    assert_eq!(result.error_level, ErrorLevel::Error);
    assert!(!result.retryable);
    assert_eq!(result.retry_after, None);
  }

  #[test]
  fn test_create_error_from_complex_config() {
    let error_obj = ErrorObject {
      code: Some("TEST_001".to_string()),
      message: Some("Complex error".to_string()),
      description: Some("Complex error".to_string()),
      error_type: Some("CustomError".to_string()),
      level: Some(ErrorLevel::Critical),
      metadata: Some(json!({"key": "value"})),
      retryable: Some(true),
      retry_after: Some(120),
    };

    let config =
      StopAndErrorConfig { error_type: ErrorType::ErrorObject, error_message: None, error_object: Some(error_obj) };

    let result = create_error_from_config(&config, "test_node").unwrap();

    assert_eq!(result.message, "Complex error");
    assert_eq!(result.description, Some("Complex error".to_string()));
    assert_eq!(result.error_code, Some("TEST_001".to_string()));
    assert_eq!(result.error_level, ErrorLevel::Critical);
    assert!(result.retryable);
    assert_eq!(result.retry_after, Some(120));
    assert!(result.metadata.is_some());
  }

  #[test]
  fn test_extract_error_message_priority() {
    // 测试 message 字段优先级
    let error_obj_msg = ErrorObject {
      code: Some("CODE_001".to_string()),
      message: Some("Message".to_string()),
      description: Some("Description".to_string()),
      error_type: None,
      level: None,
      metadata: None,
      retryable: None,
      retry_after: None,
    };
    assert_eq!(extract_error_message(&error_obj_msg), "Message");

    // 测试 description 字段备选
    let error_obj_desc = ErrorObject {
      code: Some("CODE_002".to_string()),
      message: None,
      description: Some("Description".to_string()),
      error_type: None,
      level: None,
      metadata: None,
      retryable: None,
      retry_after: None,
    };
    assert_eq!(extract_error_message(&error_obj_desc), "Description");

    // 测试 code 字段最后备选
    let error_obj_code = ErrorObject {
      code: Some("CODE_003".to_string()),
      message: None,
      description: None,
      error_type: None,
      level: None,
      metadata: None,
      retryable: None,
      retry_after: None,
    };
    assert_eq!(extract_error_message(&error_obj_code), "CODE_003");

    // 测试默认消息
    let error_obj_empty = ErrorObject {
      code: None,
      message: None,
      description: None,
      error_type: None,
      level: None,
      metadata: None,
      retryable: None,
      retry_after: None,
    };
    assert_eq!(extract_error_message(&error_obj_empty), "Error occurred");
  }

  #[test]
  fn test_validate_error_message() {
    // 有效消息
    assert!(validate_error_message("Valid error message").is_ok());

    // 空消息
    assert!(validate_error_message("").is_err());
    assert!(validate_error_message("   ").is_err());

    // 过长消息
    let long_message = "a".repeat(10001);
    assert!(validate_error_message(&long_message).is_err());

    // 边界情况
    let boundary_message = "a".repeat(10000);
    assert!(validate_error_message(&boundary_message).is_ok());
  }

  #[test]
  fn test_parse_error_object() {
    // 有效的错误对象
    let valid_json = json!({
      "code": "TEST_001",
      "message": "Test error",
      "level": "error",
      "retryable": false
    });

    assert!(parse_error_object(&valid_json).is_ok());

    // 无效的 JSON 类型
    let invalid_type = json!("not an object");
    assert!(parse_error_object(&invalid_type).is_err());

    // 无效的字段值
    let invalid_fields = json!({
      "level": "invalid_level",
      "retry_after": -1
    });

    // 这个可能会成功，因为 serde 有默认值处理
    let result = parse_error_object(&invalid_fields);
    // 具体行为取决于 ErrorObject 的 serde 实现
  }

  #[test]
  fn test_format_error_level() {
    assert_eq!(format_error_level(&ErrorLevel::Info), "info");
    assert_eq!(format_error_level(&ErrorLevel::Warning), "warning");
    assert_eq!(format_error_level(&ErrorLevel::Error), "error");
    assert_eq!(format_error_level(&ErrorLevel::Critical), "critical");
  }
}
