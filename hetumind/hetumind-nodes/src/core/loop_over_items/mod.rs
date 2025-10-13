//! Loop Over Items 数据循环节点实现
//!
//! 参考 n8n 的 Loop Over Items 节点设计，用于对数据集合进行迭代处理。
//! 支持多种循环模式，包括批量处理、条件循环、索引访问等。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod loop_v1;
mod utils;

use loop_v1::LoopV1;

use crate::constants::LOOP_OVER_ITEMS_NODE_KIND;

/// 循环模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
  /// 对每个数据项执行一次循环
  Items,
  /// 固定次数循环
  Times,
  /// 条件循环
  While,
  /// 批量处理
  Batch,
}

/// 循环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
  /// 循环模式
  pub mode: LoopMode,
  /// 循环次数（用于 Times 模式）
  pub iterations: Option<u32>,
  /// 批量大小（用于 Batch 模式）
  pub batch_size: Option<usize>,
  /// 条件表达式（用于 While 模式）
  pub condition: Option<String>,
  /// 最大循环次数（防止无限循环）
  pub max_iterations: Option<u32>,
  /// 是否包含索引
  pub include_index: bool,
  /// 是否并行处理
  pub parallel: bool,
}

impl LoopConfig {
  /// 验证循环配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    match self.mode {
      LoopMode::Times => {
        if self.iterations.is_none() || self.iterations == Some(0) {
          return Err("Iterations must be greater than 0 for Times mode".to_string());
        }
      }
      LoopMode::Batch => {
        if self.batch_size.is_none() || self.batch_size == Some(0) {
          return Err("Batch size must be greater than 0 for Batch mode".to_string());
        }
      }
      LoopMode::While => {
        if self.condition.as_ref().is_none_or(|s| s.trim().is_empty()) {
          return Err("Condition is required for While mode".to_string());
        }
      }
      LoopMode::Items => {
        // Items 模式无需额外验证
      }
    }

    // 验证最大循环次数
    if let Some(max_iter) = self.max_iterations {
      if max_iter == 0 {
        return Err("Max iterations must be greater than 0".to_string());
      }
      if max_iter > 1000000 {
        return Err("Max iterations cannot exceed 1,000,000".to_string());
      }
    }

    Ok(())
  }
}

pub struct LoopOverItemsNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl LoopOverItemsNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(LoopV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(LOOP_OVER_ITEMS_NODE_KIND, Version::new(1, 0, 0), "Loop Over Items")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description("对数据集合进行迭代处理。支持多种循环模式和批量处理。")
      .with_icon("repeat")
  }
}

impl Node for LoopOverItemsNode {
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
    let node = LoopOverItemsNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::LoopOverItems");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Loop Over Items");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = LoopOverItemsNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_loop_config_validation() {
    // 有效的 Items 配置
    let valid_items_config = LoopConfig {
      mode: LoopMode::Items,
      iterations: None,
      batch_size: None,
      condition: None,
      max_iterations: Some(100),
      include_index: false,
      parallel: false,
    };
    assert!(valid_items_config.validate().is_ok());

    // 无效的 Times 配置（没有迭代次数）
    let invalid_times_config = LoopConfig {
      mode: LoopMode::Times,
      iterations: None,
      batch_size: None,
      condition: None,
      max_iterations: None,
      include_index: false,
      parallel: false,
    };
    assert!(invalid_times_config.validate().is_err());

    // 有效的 Times 配置
    let valid_times_config = LoopConfig {
      mode: LoopMode::Times,
      iterations: Some(10),
      batch_size: None,
      condition: None,
      max_iterations: None,
      include_index: false,
      parallel: false,
    };
    assert!(valid_times_config.validate().is_ok());

    // 无效的 Batch 配置（批次大小为0）
    let invalid_batch_config = LoopConfig {
      mode: LoopMode::Batch,
      iterations: None,
      batch_size: Some(0),
      condition: None,
      max_iterations: None,
      include_index: false,
      parallel: false,
    };
    assert!(invalid_batch_config.validate().is_err());

    // 无效的 While 配置（没有条件）
    let invalid_while_config = LoopConfig {
      mode: LoopMode::While,
      iterations: None,
      batch_size: None,
      condition: None,
      max_iterations: Some(100),
      include_index: false,
      parallel: false,
    };
    assert!(invalid_while_config.validate().is_err());

    // 有效的 While 配置
    let valid_while_config = LoopConfig {
      mode: LoopMode::While,
      iterations: None,
      batch_size: None,
      condition: Some("data.enabled".to_string()),
      max_iterations: Some(1000),
      include_index: false,
      parallel: false,
    };
    assert!(valid_while_config.validate().is_ok());

    // 无效的最大迭代次数
    let invalid_max_config = LoopConfig {
      mode: LoopMode::Items,
      iterations: None,
      batch_size: None,
      condition: None,
      max_iterations: Some(0),
      include_index: false,
      parallel: false,
    };
    assert!(invalid_max_config.validate().is_err());
  }

  #[test]
  fn test_loop_mode_equality() {
    assert_eq!(LoopMode::Items, LoopMode::Items);
    assert_ne!(LoopMode::Items, LoopMode::Times);

    // 测试序列化和反序列化
    let mode = LoopMode::Batch;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: LoopMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);
  }
}
