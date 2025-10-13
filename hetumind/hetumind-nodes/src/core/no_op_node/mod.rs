//! No Operation Node (NoOp) 实现
//!
//! 参考 n8n 的 No Operation 节点设计，实现为最简单的处理节点。
//! 该节点的主要功能是原样传递输入数据，不执行任何操作或转换。
//!
//! # 功能特性
//! - 原样传递输入数据
//! - 不执行任何数据转换
//! - 最小化性能开销
//! - 支持工作流调试和组织
//! - 作为占位符节点使用

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinitionBuilder, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod no_op_v1;
mod utils;

use no_op_v1::NoOpV1;

use crate::constants::NOOP_NODE_KIND;

/// NoOp 节点配置
///
/// NoOp 节点不需要任何配置参数，因此此结构体为空。
/// 未来可以扩展用于添加日志记录、性能监控等功能。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoOpConfig {
  /// 是否启用数据传递日志
  pub enable_logging: bool,
  /// 是否记录性能指标
  pub enable_metrics: bool,
}

impl Default for NoOpConfig {
  fn default() -> Self {
    Self { enable_logging: false, enable_metrics: false }
  }
}

/// NoOp 节点
///
/// 实现 No Operation 功能，原样传递输入数据而不进行任何转换。
/// 这是工作流中最简单的节点，主要用于：
/// - 工作流调试和数据检查
/// - 工作流组织和分隔
/// - 作为占位符节点
/// - 条件分支中的空操作
pub struct NoOpNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl NoOpNode {
  /// 创建新的 NoOp 节点实例
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(NoOpV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  /// 创建基础节点定义构建器
  fn base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(NodeKind::from(NOOP_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform])
      .display_name("No Operation")
      .description("Pass through data without any modifications")
      .icon("arrow-right")
      .version(Version::new(1, 0, 0));
    base
  }
}

impl Node for NoOpNode {
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
    let node = NoOpNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::NoOp");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform]);
    assert_eq!(&definition.display_name, "No Operation");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = NoOpNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_noop_config_default() {
    let config = NoOpConfig::default();
    assert!(!config.enable_logging);
    assert!(!config.enable_metrics);
  }

  #[test]
  fn test_noop_config_serialization() {
    let config = NoOpConfig { enable_logging: true, enable_metrics: false };

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: NoOpConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(config.enable_logging, deserialized.enable_logging);
    assert_eq!(config.enable_metrics, deserialized.enable_metrics);
  }
}
