//! Set 数据设置节点实现
//!
//! 参考 n8n 的 Set 节点设计，用于设置、修改或删除数据字段。
//! 支持多种操作类型和数据源，是数据处理工作流中的重要节点。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinitionBuilder, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod set_v1;
mod utils;

use set_v1::SetV1;

use crate::constants::SET_NODE_KIND;

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
  /// 设置字段值
  Set,
  /// 删除字段
  Remove,
  /// 从其他字段复制值
  Copy,
  /// 数值增加
  Increment,
  /// 数组追加元素
  Append,
}

/// 数据来源类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueSourceKind {
  /// 静态值
  Static,
  /// 表达式（JSON Path）
  Expression,
  /// 当前时间戳
  CurrentTimestamp,
  /// 随机值
  Random,
}

/// 设置操作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetOperation {
  /// 目标字段路径
  pub field_path: String,
  /// 操作类型
  pub kind: OperationKind,
  /// 值来源类型
  pub value_source: ValueSourceKind,
  /// 设置的值（当 value_source 为 StaticValue 或 Expression 时使用）
  pub value: Option<Value>,
  /// 是否保留原始类型
  pub keep: Option<bool>,
}

impl SetOperation {
  /// 验证操作配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.field_path.trim().is_empty() {
      return Err("Field path cannot be empty".to_string());
    }

    match self.kind {
      OperationKind::Set | OperationKind::Copy | OperationKind::Increment | OperationKind::Append => {
        if self.value_source == ValueSourceKind::Static && self.value.is_none() {
          return Err("Static value source requires a value".to_string());
        }
        if self.value_source == ValueSourceKind::Expression
          && self.value.as_ref().and_then(|v| v.as_str()).is_none_or(|s| s.trim().is_empty())
        {
          return Err("Expression value source requires a valid expression".to_string());
        }
      }
      OperationKind::Remove => {
        // Remove 操作不需要值
      }
    }

    Ok(())
  }
}

pub struct SetNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl SetNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(SetV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(NodeKind::from(SET_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output])
      .display_name("Set")
      .description("设置、修改或删除数据字段。支持多种操作类型和数据来源。")
      .icon("edit")
      .version(Version::new(1, 0, 0));
    base
  }
}

impl Node for SetNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
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
    let node = SetNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "Set");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Set");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = SetNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_operation_validation() {
    // 有效的设置操作
    let valid_set_op = SetOperation {
      field_path: "user.name".to_string(),
      kind: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: Some(Value::String("John".to_string())),
      keep: None,
    };
    assert!(valid_set_op.validate().is_ok());

    // 无效的空字段路径
    let invalid_path_op = SetOperation {
      field_path: "".to_string(),
      kind: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: Some(Value::String("test".to_string())),
      keep: None,
    };
    assert!(invalid_path_op.validate().is_err());

    // 无效的静态值来源
    let invalid_static_op = SetOperation {
      field_path: "test".to_string(),
      kind: OperationKind::Set,
      value_source: ValueSourceKind::Static,
      value: None,
      keep: None,
    };
    assert!(invalid_static_op.validate().is_err());

    // 有效的删除操作（不需要值）
    let valid_remove_op = SetOperation {
      field_path: "user.temp".to_string(),
      kind: OperationKind::Remove,
      value_source: ValueSourceKind::Static,
      value: None,
      keep: None,
    };
    assert!(valid_remove_op.validate().is_ok());
  }
}
