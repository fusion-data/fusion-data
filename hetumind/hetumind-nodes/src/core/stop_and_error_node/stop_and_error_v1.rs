use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition, NodeDefinitionBuilder,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, RegistrationError,
  },
};
use serde_json::json;

use super::{
  ErrorType, StopAndErrorConfig,
  utils::{create_error_from_config, format_error_level, validate_config_with_context},
};

/// Stop And Error 节点 V1
///
/// 用于在工作流执行过程中主动抛出错误以终止工作流。
/// 支持简单错误消息和复杂错误对象两种模式。
///
/// # 错误类型
/// - `ErrorMessage`: 简单字符串错误消息
/// - `ErrorObject`: 复杂结构化错误对象，包含代码、级别、元数据等
///
/// # 特性
/// - 无输出端口：强制终止工作流执行
/// - 支持元数据：错误对象可包含丰富的上下文信息
/// - 重试控制：可配置错误是否可重试及重试延迟
/// - 级别分类：支持 Info、Warning、Error、Critical 四个级别
///
/// # 输入
/// - 接收任意 JSON 数据，但不会传递到输出
///
/// # 输出
/// - 无输出端口，总是抛出错误
#[derive(Debug)]
pub struct StopAndErrorV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl NodeExecutable for StopAndErrorV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Stop And Error 节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据（仅用于验证和上下文）
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Stop And Error 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::warn!("Stop And Error 节点没有接收到输入数据，将使用默认配置");
      Vec::new()
    };

    // 获取错误配置
    let error_type: ErrorType = node.get_parameter("error_type").unwrap_or_default();
    let error_message: Option<String> = node.get_optional_parameter("error_message");
    let error_object: Option<serde_json::Value> = node.get_optional_parameter("error_object");

    let config = StopAndErrorConfig {
      error_type,
      error_message,
      error_object: error_object.and_then(|value| serde_json::from_value(value).ok()),
    };

    // 验证配置
    if let Err(e) = validate_config_with_context(&config, &context.current_node_name) {
      log::error!("Stop And Error 配置验证失败: {}", e);
      return Err(NodeExecutionError::ConfigurationError(format!("Invalid Stop and Error configuration: {}", e)));
    }

    log::debug!("错误配置: 类型={:?}, 消息={:?}", config.error_type, config.get_error_message());

    // 创建错误结果
    let error_result = create_error_from_config(&config, &context.current_node_name)?;

    log::info!(
      "Stop And Error 节点执行完成: 错误级别={:?}, 错误代码={:?}, 可重试={}",
      config.get_error_level(),
      config.get_error_code(),
      config.is_retryable()
    );

    // 构建错误消息，包含所有相关信息
    let mut error_message = error_result.message;
    if let Some(description) = error_result.description {
      error_message = format!("{}: {}", error_message, description);
    }
    if let Some(code) = error_result.error_code {
      error_message = format!("[{}] {}", code, error_message);
    }

    // 添加级别信息
    let level_str = format_error_level(&error_result.error_level);
    error_message = format!("({}) {}", level_str, error_message);

    // 添加重试信息
    if error_result.retryable {
      if let Some(retry_after) = error_result.retry_after {
        error_message = format!("{} [Retryable after {}s]", error_message, retry_after);
      } else {
        error_message = format!("{} [Retryable]", error_message);
      }
    }

    // 抛出工作流执行错误
    Err(NodeExecutionError::ExecutionFailed {
      node_name: context.current_node_name.clone(),
      message: Some(error_message),
    })
  }
}

impl TryFrom<NodeDefinitionBuilder> for StopAndErrorV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .version(Version::new(1, 0, 0))
      .inputs([InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
      .outputs([]) // Stop and Error 节点没有输出端口
      .properties([
        NodeProperty::builder()
          .display_name("错误类型")
          .name("error_type")
          .kind(NodePropertyKind::Options)
          .required(true)
          .description("选择错误类型")
          .value(json!(ErrorType::ErrorMessage))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "错误消息",
              "error_message",
              json!(ErrorType::ErrorMessage),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "错误对象",
              "error_object",
              json!(ErrorType::ErrorObject),
              NodePropertyKind::String,
            )),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("错误消息")
          .name("error_message")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("要抛出的错误消息（当错误类型为「错误消息」时使用）")
          .placeholder("输入错误消息...")
          .build(),
        NodeProperty::builder()
          .display_name("错误对象")
          .name("error_object")
          .kind(NodePropertyKind::Json)
          .required(false)
          .description("结构化错误对象（当错误类型为「错误对象」时使用）")
          .placeholder("输入 JSON 格式的错误对象...")
          .build(),
      ]);

    let definition = base.build()?;

    Ok(Self { definition: Arc::new(definition) })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::StopAndErrorNode;
  use hetumind_core::workflow::Node;

  #[test]
  fn test_node_definition_properties() {
    // 使用 StopAndErrorNode
    let node = StopAndErrorNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    // 验证基本属性
    assert_eq!(definition.version, Version::new(1, 0, 0));
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 0); // 无输出端口

    // 验证属性配置
    let error_type_prop = definition.properties.iter().find(|p| p.name == "error_type");
    assert!(error_type_prop.is_some());
    assert!(error_type_prop.unwrap().required);

    let error_message_prop = definition.properties.iter().find(|p| p.name == "error_message");
    assert!(error_message_prop.is_some());
    assert!(!error_message_prop.unwrap().required);

    let error_object_prop = definition.properties.iter().find(|p| p.name == "error_object");
    assert!(error_object_prop.is_some());
    assert!(!error_object_prop.unwrap().required);
  }
}
