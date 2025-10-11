//! Switch 条件路由节点实现
//!
//! 参考 n8n 的 Switch 节点设计，用于根据条件或表达式将输入数据路由到不同的输出端口。
//! 支持两种工作模式：Rules（规则）模式和 Expression（表达式）模式。

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinitionBuilder, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError, ValidationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::r#if::ConditionConfig;

mod switch_v1;
mod utils;

use switch_v1::SwitchV1;

use crate::constants::SWITCH_NODE_KIND;

/// Switch 工作模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwitchMode {
  /// 基于规则集合的路由模式
  Rules,
  /// 基于表达式计算的路由模式
  Expression,
}

/// Fallback 输出策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FallbackOutput {
  /// 无 fallback，忽略不匹配的数据
  None,
  /// 输出到额外的备用端口
  Extra,
  /// 输出到指定端口
  Port(usize),
}

/// Switch 规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchRule {
  /// 规则名称/输出键
  pub output_key: Option<String>,
  /// 规则条件集合
  pub conditions: Vec<ConditionConfig>,
  /// 输出端口索引
  pub output_index: Option<usize>,
}

/// Switch 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchConfig {
  /// 工作模式
  pub mode: SwitchMode,
  /// 规则集合（Rules 模式）
  pub rules: Option<Vec<SwitchRule>>,
  /// 输出端口数量（Expression 模式）
  pub number_outputs: Option<usize>,
  /// 输出表达式（Expression 模式）
  pub output_expression: Option<Value>,
  /// 选项配置
  pub options: SwitchOptions,
}

/// Switch 选项配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchOptions {
  /// 是否允许输出到所有匹配的端口
  pub all_matching_outputs: Option<bool>,
  /// 是否忽略大小写
  pub ignore_case: Option<bool>,
  /// 是否使用宽松类型验证
  pub loose_type_validation: Option<bool>,
  /// Fallback 输出策略
  pub fallback_output: Option<FallbackOutput>,
}

impl Default for SwitchOptions {
  fn default() -> Self {
    Self {
      all_matching_outputs: Some(false),
      ignore_case: Some(false),
      loose_type_validation: Some(false),
      fallback_output: Some(FallbackOutput::None),
    }
  }
}

impl SwitchConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    match self.mode {
      SwitchMode::Rules => {
        if self.rules.is_none() || self.rules.as_ref().unwrap().is_empty() {
          return Err(ValidationError::invalid_field_value(
            "rules".to_string(),
            "At least one rule is required for Rules mode".to_string(),
          ));
        }
      }
      SwitchMode::Expression => {
        if self.number_outputs.is_none() || self.number_outputs.unwrap() == 0 {
          return Err(ValidationError::invalid_field_value(
            "number_outputs".to_string(),
            "Number of outputs must be greater than 0 for Expression mode".to_string(),
          ));
        }
        if self.output_expression.is_none() {
          return Err(ValidationError::invalid_field_value(
            "output_expression".to_string(),
            "Output expression is required for Expression mode".to_string(),
          ));
        }
      }
    }
    Ok(())
  }
}

pub struct SwitchNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl SwitchNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(SwitchV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(NodeKind::from(SWITCH_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output])
      .display_name("Switch")
      .description(
        "Routes input data to different output ports based on conditions or expressions. Supports both rules-based and expression-based routing.",
      )
      .icon("code-branch")
      .version(Version::new(1, 0, 0));
    base
  }
}

impl Node for SwitchNode {
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
  use hetumind_core::workflow::NodeGroupKind;

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = SwitchNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::Switch");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Switch");
  }

  #[test]
  fn test_switch_config_validation() {
    // 有效的 Rules 配置
    let valid_rules_config = SwitchConfig {
      mode: SwitchMode::Rules,
      rules: Some(vec![SwitchRule {
        output_key: Some("output1".to_string()),
        conditions: vec![],
        output_index: Some(0),
      }]),
      number_outputs: None,
      output_expression: None,
      options: SwitchOptions::default(),
    };
    assert!(valid_rules_config.validate().is_ok());

    // 无效的 Rules 配置（没有规则）
    let invalid_rules_config = SwitchConfig {
      mode: SwitchMode::Rules,
      rules: Some(vec![]),
      number_outputs: None,
      output_expression: None,
      options: SwitchOptions::default(),
    };
    assert!(invalid_rules_config.validate().is_err());

    // 有效的 Expression 配置
    let valid_expression_config = SwitchConfig {
      mode: SwitchMode::Expression,
      rules: None,
      number_outputs: Some(3),
      output_expression: Some(serde_json::json!("{{ $json.index }}")),
      options: SwitchOptions::default(),
    };
    assert!(valid_expression_config.validate().is_ok());

    // 无效的 Expression 配置（没有输出数量）
    let invalid_expression_config = SwitchConfig {
      mode: SwitchMode::Expression,
      rules: None,
      number_outputs: Some(0),
      output_expression: Some(serde_json::json!("{{ $json.index }}")),
      options: SwitchOptions::default(),
    };
    assert!(invalid_expression_config.validate().is_err());
  }

  #[test]
  fn test_switch_mode_equality() {
    assert_eq!(SwitchMode::Rules, SwitchMode::Rules);
    assert_ne!(SwitchMode::Rules, SwitchMode::Expression);

    // 测试序列化和反序列化
    let mode = SwitchMode::Expression;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: SwitchMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);
  }

  #[test]
  fn test_fallback_output() {
    // 测试序列化和反序列化
    let fallback = FallbackOutput::Port(2);
    let serialized = serde_json::to_string(&fallback).unwrap();
    let deserialized: FallbackOutput = serde_json::from_str(&serialized).unwrap();
    assert_eq!(fallback, deserialized);

    let fallback_none = FallbackOutput::None;
    let serialized_none = serde_json::to_string(&fallback_none).unwrap();
    let deserialized_none: FallbackOutput = serde_json::from_str(&serialized_none).unwrap();
    assert_eq!(fallback_none, deserialized_none);
  }
}
