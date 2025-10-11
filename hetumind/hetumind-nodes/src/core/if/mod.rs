//! If 条件判断节点实现
//!
//! 参考 n8n 的 If 节点设计，用于根据条件判断分割工作流执行路径。
//! 支持多种数据类型的比较操作，包括字符串、数字、布尔值、日期时间等。

use std::sync::Arc;

use hetumind_core::{
  types::DataType,
  version::Version,
  workflow::{Node, NodeDefinitionBuilder, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError, ValidationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod if_v1;
pub mod utils;

use if_v1::IfV1;

use crate::constants::IF_NODE_KIND;

/// 条件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionConfig {
  /// 左值（通常是表达式）
  pub left: Value,
  /// 比较操作
  pub op: ComparisonOperation,
  /// 右值（比较目标）
  pub right: Option<Value>,
  /// 数据类型
  pub data_type: DataType,
}

impl ConditionConfig {
  /// Verify if the condition configuration is valid.
  #[allow(dead_code)]
  pub fn validate(&self) -> Result<(), ValidationError> {
    // 非[相等或不相等]时，右值不可为空
    if !(self.op == ComparisonOperation::Eq || self.op == ComparisonOperation::Nq) && self.right.is_none() {
      return Err(ValidationError::invalid_field_value("right".to_string(), "Cannot be empty".to_string()));
    }

    Ok(())
  }
}

/// 比较操作类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperation {
  /// Equal
  Eq,
  /// Not Equal
  Nq,
  /// Contains
  Contains,
  /// Not Contains
  NotContains,
  /// Start With
  StartsWith,
  /// End With
  EndsWith,
  /// Regex
  Regex,
  /// Empty
  Empty,
  /// Not Empty
  NotEmpty,
  /// Greater Than
  Gt,
  /// Less Than
  Lt,
  /// Greater Than or Equal
  Gte,
  /// Less Than or Equal
  Lte,
  /// Is True
  Is,
  /// Is False
  Not,
}

/// 逻辑组合方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicCombination {
  And,
  Or,
}

pub struct IfNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl IfNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(IfV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(NodeKind::from(IF_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output])
      .display_name("If")
      .description(
        "Splits workflow execution paths based on conditions. Supports comparison operations for multiple data types.",
      )
      .icon("code-branch")
      .version(Version::new(1, 0, 0));
    base
  }
}

impl Node for IfNode {
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
    let node = IfNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::If");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "If");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 2);
  }

  #[test]
  fn test_node_ports() {
    let node = IfNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 2);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
    assert_eq!(output_ports[1].kind, ConnectionKind::Main);
  }
}
