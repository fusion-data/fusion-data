//! Stop And Error 节点实现
//!
//! 参考 n8n 的 Stop and Error 节点设计，用于在工作流执行过程中主动抛出错误。
//! 支持简单错误消息和复杂错误对象两种模式，是工作流错误控制的重要组成部分。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeDefinition, NodeGroupKind, NodeKind, RegistrationError, ValidationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod stop_and_error_v1;
mod utils;

use stop_and_error_v1::StopAndErrorV1;

use crate::constants::STOP_AND_ERROR_NODE_KIND;

/// 错误类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
  /// 简单错误消息
  ErrorMessage,
  /// 复杂错误对象
  ErrorObject,
}

/// 错误对象配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
  /// 错误代码
  pub code: Option<String>,
  /// 错误消息
  pub message: Option<String>,
  /// 错误描述
  pub description: Option<String>,
  /// 错误类型
  pub error_type: Option<String>,
  /// 错误级别
  pub level: Option<ErrorLevel>,
  /// 元数据
  pub metadata: Option<Value>,
  /// 是否可重试
  pub retryable: Option<bool>,
  /// 重试延迟（秒）
  pub retry_after: Option<u64>,
}

/// 错误级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorLevel {
  /// 信息
  Info,
  /// 警告
  Warning,
  /// 错误
  Error,
  /// 严重错误
  Critical,
}

/// Stop And Error 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopAndErrorConfig {
  /// 错误类型
  pub error_type: ErrorType,
  /// 简单错误消息（当 error_type 为 ErrorMessage 时使用）
  pub error_message: Option<String>,
  /// 复杂错误对象（当 error_type 为 ErrorObject 时使用）
  pub error_object: Option<ErrorObject>,
}

impl StopAndErrorConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    match self.error_type {
      ErrorType::ErrorMessage => {
        if self.error_message.as_ref().is_none_or(|s| s.trim().is_empty()) {
          return Err(ValidationError::invalid_field_value(
            "error_message".to_string(),
            "Error message cannot be empty".to_string(),
          ));
        }
      }
      ErrorType::ErrorObject => {
        if self.error_object.is_none() {
          return Err(ValidationError::invalid_field_value(
            "error_object".to_string(),
            "Error object is required when error_type is 'error_object'".to_string(),
          ));
        }

        if let Some(ref error_obj) = self.error_object
          && error_obj.message.as_ref().is_none_or(|s| s.trim().is_empty())
          && error_obj.description.as_ref().is_none_or(|s| s.trim().is_empty())
          && error_obj.code.as_ref().is_none_or(|s| s.trim().is_empty())
        {
          return Err(ValidationError::invalid_field_value(
            "error_object".to_string(),
            "Error object must have at least one of: message, description, or code".to_string(),
          ));
        }
      }
    }

    Ok(())
  }

  /// 获取有效的错误消息
  pub fn get_error_message(&self) -> String {
    match &self.error_type {
      ErrorType::ErrorMessage => self.error_message.clone().unwrap_or_else(|| "Error occurred".to_string()),
      ErrorType::ErrorObject => {
        if let Some(ref error_obj) = self.error_object {
          error_obj
            .message
            .clone()
            .or_else(|| error_obj.description.clone())
            .or_else(|| error_obj.code.clone())
            .unwrap_or_else(|| "Error occurred".to_string())
        } else {
          "Error occurred".to_string()
        }
      }
    }
  }

  /// 获取错误描述
  pub fn get_error_description(&self) -> Option<String> {
    match &self.error_type {
      ErrorType::ErrorMessage => self.error_message.clone(),
      ErrorType::ErrorObject => self
        .error_object
        .as_ref()
        .and_then(|obj| obj.message.clone().or_else(|| obj.description.clone()).or_else(|| obj.code.clone())),
    }
  }

  /// 获取错误代码
  pub fn get_error_code(&self) -> Option<String> {
    match &self.error_type {
      ErrorType::ErrorMessage => None,
      ErrorType::ErrorObject => self.error_object.as_ref().and_then(|obj| obj.code.clone()),
    }
  }

  /// 获取错误级别
  pub fn get_error_level(&self) -> ErrorLevel {
    match &self.error_type {
      ErrorType::ErrorMessage => ErrorLevel::Error,
      ErrorType::ErrorObject => {
        self.error_object.as_ref().and_then(|obj| obj.level.clone()).unwrap_or(ErrorLevel::Error)
      }
    }
  }

  /// 获取错误元数据
  pub fn get_error_metadata(&self) -> Option<Value> {
    match &self.error_type {
      ErrorType::ErrorMessage => Some(serde_json::json!({
        "error_type": "simple_message",
        "message": self.error_message
      })),
      ErrorType::ErrorObject => self.error_object.as_ref().map(|obj| {
        let mut metadata = obj.metadata.clone().unwrap_or(Value::Object(Default::default()));

        if let Value::Object(ref mut map) = metadata {
          if let Some(ref code) = obj.code {
            map.insert("code".to_string(), Value::String(code.clone()));
          }
          if let Some(ref error_type) = obj.error_type {
            map.insert("error_type".to_string(), Value::String(error_type.clone()));
          }
          if let Some(ref level) = obj.level {
            map.insert("level".to_string(), serde_json::to_value(level).unwrap_or(Value::Null));
          }
          if obj.retryable.is_some() {
            map.insert("retryable".to_string(), serde_json::to_value(obj.retryable).unwrap_or(Value::Null));
          }
          if let Some(retry_after) = obj.retry_after {
            map.insert("retry_after".to_string(), Value::Number(serde_json::Number::from(retry_after)));
          }
        }

        metadata
      }),
    }
  }

  /// 检查错误是否可重试
  pub fn is_retryable(&self) -> bool {
    match &self.error_type {
      ErrorType::ErrorMessage => false,
      ErrorType::ErrorObject => self.error_object.as_ref().and_then(|obj| obj.retryable).unwrap_or(false),
    }
  }

  /// 获取重试延迟（秒）
  pub fn get_retry_after(&self) -> Option<u64> {
    match &self.error_type {
      ErrorType::ErrorMessage => None,
      ErrorType::ErrorObject => self.error_object.as_ref().and_then(|obj| obj.retry_after),
    }
  }
}

impl Default for StopAndErrorConfig {
  fn default() -> Self {
    Self { error_type: ErrorType::ErrorMessage, error_message: Some("Error occurred".to_string()), error_object: None }
  }
}

#[allow(clippy::derivable_impls)]
impl Default for ErrorType {
  fn default() -> Self {
    Self::ErrorMessage
  }
}

#[allow(clippy::derivable_impls)]
impl Default for ErrorLevel {
  fn default() -> Self {
    Self::Error
  }
}

pub struct StopAndErrorNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl StopAndErrorNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(StopAndErrorV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(STOP_AND_ERROR_NODE_KIND, "Stop and Error")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .with_description("主动抛出错误以终止工作流执行。支持简单错误消息和复杂错误对象。")
      .with_icon("exclamation-triangle")
  }
}

impl Node for StopAndErrorNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = StopAndErrorNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::StopAndError");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input]);
    assert_eq!(&definition.display_name, "Stop and Error");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 0); // 无输出端口
  }

  #[test]
  fn test_node_ports() {
    let node = StopAndErrorNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 0); // Stop and Error 节点没有输出
  }

  #[test]
  fn test_error_type_equality() {
    assert_eq!(ErrorType::ErrorMessage, ErrorType::ErrorMessage);
    assert_ne!(ErrorType::ErrorMessage, ErrorType::ErrorObject);

    // 测试序列化和反序列化
    let error_type = ErrorType::ErrorObject;
    let serialized = serde_json::to_string(&error_type).unwrap();
    let deserialized: ErrorType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(error_type, deserialized);
  }

  #[test]
  fn test_error_level_equality() {
    assert_eq!(ErrorLevel::Error, ErrorLevel::Error);
    assert_ne!(ErrorLevel::Error, ErrorLevel::Critical);

    // 测试序列化和反序列化
    let error_level = ErrorLevel::Warning;
    let serialized = serde_json::to_string(&error_level).unwrap();
    let deserialized: ErrorLevel = serde_json::from_str(&serialized).unwrap();
    assert_eq!(error_level, deserialized);
  }

  #[test]
  fn test_config_validation() {
    // 有效的简单错误配置
    let valid_simple_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorMessage,
      error_message: Some("Test error".to_string()),
      error_object: None,
    };
    assert!(valid_simple_config.validate().is_ok());

    // 无效的简单错误配置（空消息）
    let invalid_simple_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorMessage,
      error_message: Some("".to_string()),
      error_object: None,
    };
    assert!(invalid_simple_config.validate().is_err());

    // 有效的复杂错误配置
    let valid_complex_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorObject,
      error_message: None,
      error_object: Some(ErrorObject {
        code: Some("TEST_001".to_string()),
        message: Some("Test error".to_string()),
        description: None,
        error_type: Some("TestError".to_string()),
        level: Some(ErrorLevel::Error),
        metadata: None,
        retryable: Some(false),
        retry_after: None,
      }),
    };
    assert!(valid_complex_config.validate().is_ok());

    // 无效的复杂错误配置（缺少错误对象）
    let invalid_complex_config =
      StopAndErrorConfig { error_type: ErrorType::ErrorObject, error_message: None, error_object: None };
    assert!(invalid_complex_config.validate().is_err());

    // 无效的复杂错误配置（空错误对象）
    let invalid_empty_complex_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorObject,
      error_message: None,
      error_object: Some(ErrorObject {
        code: None,
        message: None,
        description: None,
        error_type: None,
        level: None,
        metadata: None,
        retryable: None,
        retry_after: None,
      }),
    };
    assert!(invalid_empty_complex_config.validate().is_err());
  }

  #[test]
  fn test_error_message_extraction() {
    // 简单错误消息
    let simple_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorMessage,
      error_message: Some("Simple error".to_string()),
      error_object: None,
    };
    assert_eq!(simple_config.get_error_message(), "Simple error");
    assert_eq!(simple_config.get_error_description(), Some("Simple error".to_string()));
    assert_eq!(simple_config.get_error_code(), None);

    // 复杂错误对象 - 优先使用 message
    let complex_config_msg = StopAndErrorConfig {
      error_type: ErrorType::ErrorObject,
      error_message: None,
      error_object: Some(ErrorObject {
        code: Some("CODE_001".to_string()),
        message: Some("Error message".to_string()),
        description: Some("Error description".to_string()),
        error_type: Some("CustomError".to_string()),
        level: Some(ErrorLevel::Critical),
        metadata: Some(serde_json::json!({"key": "value"})),
        retryable: Some(true),
        retry_after: Some(60),
      }),
    };
    assert_eq!(complex_config_msg.get_error_message(), "Error message");
    assert_eq!(complex_config_msg.get_error_description(), Some("Error message".to_string()));
    assert_eq!(complex_config_msg.get_error_code(), Some("CODE_001".to_string()));
    assert_eq!(complex_config_msg.get_error_level(), ErrorLevel::Critical);
    assert!(complex_config_msg.is_retryable());
    assert_eq!(complex_config_msg.get_retry_after(), Some(60));

    // 复杂错误对象 - 使用 description 作为备选
    let complex_config_desc = StopAndErrorConfig {
      error_type: ErrorType::ErrorObject,
      error_message: None,
      error_object: Some(ErrorObject {
        code: Some("CODE_002".to_string()),
        message: None,
        description: Some("Error description".to_string()),
        error_type: None,
        level: None,
        metadata: None,
        retryable: None,
        retry_after: None,
      }),
    };
    assert_eq!(complex_config_desc.get_error_message(), "Error description");
    assert_eq!(complex_config_desc.get_error_description(), Some("Error description".to_string()));

    // 复杂错误对象 - 使用 code 作为最后备选
    let complex_config_code = StopAndErrorConfig {
      error_type: ErrorType::ErrorObject,
      error_message: None,
      error_object: Some(ErrorObject {
        code: Some("CODE_003".to_string()),
        message: None,
        description: None,
        error_type: None,
        level: None,
        metadata: None,
        retryable: None,
        retry_after: None,
      }),
    };
    assert_eq!(complex_config_code.get_error_message(), "CODE_003");
    assert_eq!(complex_config_code.get_error_description(), Some("CODE_003".to_string()));
  }

  #[test]
  fn test_default_config() {
    let default_config = StopAndErrorConfig::default();
    assert_eq!(default_config.error_type, ErrorType::ErrorMessage);
    assert_eq!(default_config.error_message, Some("Error occurred".to_string()));
    assert!(default_config.error_object.is_none());
  }

  #[test]
  fn test_error_metadata() {
    // 简单错误的元数据
    let simple_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorMessage,
      error_message: Some("Simple error".to_string()),
      error_object: None,
    };
    let simple_metadata = simple_config.get_error_metadata().unwrap();
    assert_eq!(simple_metadata["error_type"], "simple_message");
    assert_eq!(simple_metadata["message"], "Simple error");

    // 复杂错误的元数据
    let complex_config = StopAndErrorConfig {
      error_type: ErrorType::ErrorObject,
      error_message: None,
      error_object: Some(ErrorObject {
        code: Some("TEST_001".to_string()),
        message: None,
        description: None,
        error_type: Some("CustomError".to_string()),
        level: Some(ErrorLevel::Warning),
        metadata: Some(serde_json::json!({"custom": "data"})),
        retryable: Some(true),
        retry_after: Some(30),
      }),
    };
    let complex_metadata = complex_config.get_error_metadata().unwrap();
    assert_eq!(complex_metadata["code"], "TEST_001");
    assert_eq!(complex_metadata["error_type"], "CustomError");
    assert_eq!(complex_metadata["level"], "warning");
    assert_eq!(complex_metadata["retryable"], true);
    assert_eq!(complex_metadata["retry_after"], 30);
    assert_eq!(complex_metadata["custom"], "data");
  }
}
