//! Limit 节点实现
//!
//! 参考 n8n 的 Limit 节点设计，用于限制通过的数据项数量。
//! 支持保留前 N 个或后 N 个数据项，提供简单而高效的数据流控制。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError, ValidationError},
};
use serde::{Deserialize, Serialize};

mod limit_v1;
pub mod utils;

#[cfg(test)]
mod tests;

use limit_v1::LimitV1;

use crate::constants::LIMIT_NODE_KIND;

/// 保留策略
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeepStrategy {
  /// 保留前 N 个项目
  #[default]
  FirstItems,
  /// 保留后 N 个项目
  LastItems,
}

/// Limit 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitConfig {
  /// 最大项目数量
  pub max_items: usize,
  /// 保留策略
  pub keep_strategy: KeepStrategy,
  /// 是否在超过限制时记录警告
  pub warn_on_limit: bool,
}

impl Default for LimitConfig {
  fn default() -> Self {
    Self { max_items: 1, keep_strategy: KeepStrategy::FirstItems, warn_on_limit: true }
  }
}

impl LimitConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.max_items == 0 {
      return Err(ValidationError::invalid_field_value(
        "max_items".to_string(),
        "max_items must be greater than 0".to_string(),
      ));
    }
    Ok(())
  }

  /// 获取配置描述
  #[allow(dead_code)]
  pub fn get_description(&self) -> String {
    format!(
      "Limit to {} {} items",
      self.max_items,
      match self.keep_strategy {
        KeepStrategy::FirstItems => "first",
        KeepStrategy::LastItems => "last",
      }
    )
  }
}

pub struct LimitNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl LimitNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(LimitV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(LIMIT_NODE_KIND, "Limit")
      .add_group(NodeGroupKind::Transform)
      .with_description("Restrict the number of items that pass through. Keeps first or last N items.")
      .with_icon("scissors")
  }
}

impl Node for LimitNode {
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
