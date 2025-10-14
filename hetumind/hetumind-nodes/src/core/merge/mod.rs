//! Merge 数据合并节点实现
//!
//! 参考 n8n 的 Merge 节点设计，用于合并多个分支的数据流。
//! 支持多种合并模式，包括简单追加、按键合并、按索引合并等。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod merge_v1;
mod utils;

use merge_v1::MergeV1;

use crate::constants::MERGE_NODE_KIND;

/// 合并模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeMode {
  /// 简单追加：将所有输入数据按顺序合并
  Append,
  /// 按键合并：根据指定字段合并相同键的数据
  MergeByKey,
  /// 按索引合并：相同索引位置的数据合并
  MergeByIndex,
  /// 等待全部：确保所有输入分支都有数据后再合并
  WaitForAll,
}

/// 合并配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConfig {
  /// 合并模式
  pub mode: MergeMode,
  /// 合并键（用于 MergeByKey 模式）
  pub merge_key: Option<String>,
  /// 期望的输入端口数量
  pub input_ports: Option<usize>,
}

impl MergeConfig {
  /// 验证合并配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    match self.mode {
      MergeMode::MergeByKey => {
        if self.merge_key.as_ref().map_or(true, |s| s.trim().is_empty()) {
          return Err("Merge key is required for MergeByKey mode".to_string());
        }
      }
      MergeMode::WaitForAll => {
        if let Some(ports) = self.input_ports {
          if ports < 2 || ports > 10 {
            return Err("Input ports must be between 2 and 10".to_string());
          }
        }
      }
      _ => {}
    }
    Ok(())
  }
}

pub struct MergeNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl MergeNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(MergeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(MERGE_NODE_KIND, Version::new(1, 0, 0), "Merge")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description("合并多个分支的数据流。支持多种合并策略。")
      .with_icon("git-merge")
  }
}

impl Node for MergeNode {
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
    let node = MergeNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::Merge");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Merge");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = MergeNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_merge_config_validation() {
    // 有效的 Append 配置
    let valid_append_config = MergeConfig { mode: MergeMode::Append, merge_key: None, input_ports: None };
    assert!(valid_append_config.validate().is_ok());

    // 无效的 MergeByKey 配置（缺少合并键）
    let invalid_merge_by_key_config = MergeConfig { mode: MergeMode::MergeByKey, merge_key: None, input_ports: None };
    assert!(invalid_merge_by_key_config.validate().is_err());

    // 无效的 WaitForAll 配置（端口数量超出范围）
    let invalid_wait_for_all_config =
      MergeConfig { mode: MergeMode::WaitForAll, merge_key: None, input_ports: Some(15) };
    assert!(invalid_wait_for_all_config.validate().is_err());

    // 有效的 MergeByKey 配置
    let valid_merge_by_key_config =
      MergeConfig { mode: MergeMode::MergeByKey, merge_key: Some("id".to_string()), input_ports: None };
    assert!(valid_merge_by_key_config.validate().is_ok());
  }

  #[test]
  fn test_merge_mode_equality() {
    assert_eq!(MergeMode::Append, MergeMode::Append);
    assert_ne!(MergeMode::Append, MergeMode::MergeByKey);

    // 测试序列化和反序列化
    let mode = MergeMode::MergeByKey;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: MergeMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);
  }
}
