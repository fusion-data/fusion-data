//! Split Out Node 实现
//!
//! 参考 n8n 的 Split Out 节点设计，将输入项中的数组或对象字段拆分为多个独立的输出项。
//! 支持多种包含策略、字段映射和二进制数据处理。
//!
//! # 功能特性
//! - 将嵌套数组/对象转换为独立数据项
//! - 支持多字段同时拆分
//! - 三种字段包含策略：noOtherFields、allOtherFields、selectedOtherFields
//! - 支持字段重命名映射
//! - 二进制数据特殊处理
//! - 点记号字段路径支持
//! - 完善的错误处理和执行提示

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinitionBuilder, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod split_out_v1;
mod utils;

use split_out_v1::SplitOutV1;

use crate::constants::SPLIT_OUT_NODE_KIND;

/// 字段包含策略枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncludeStrategy {
  /// 仅保留拆分字段
  NoOtherFields,
  /// 保留所有其他字段
  AllOtherFields,
  /// 选择性保留其他字段
  SelectedOtherFields,
}

impl Default for IncludeStrategy {
  fn default() -> Self {
    Self::NoOtherFields
  }
}

impl std::fmt::Display for IncludeStrategy {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      IncludeStrategy::NoOtherFields => write!(f, "noOtherFields"),
      IncludeStrategy::AllOtherFields => write!(f, "allOtherFields"),
      IncludeStrategy::SelectedOtherFields => write!(f, "selectedOtherFields"),
    }
  }
}

/// 拆分字段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldToSplit {
  /// 要拆分的字段路径
  pub field_to_split: String,
  /// 目标字段名称（可选）
  pub destination_field: Option<String>,
}

impl FieldToSplit {
  /// 验证字段配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.field_to_split.trim().is_empty() {
      return Err("Field to split cannot be empty".to_string());
    }

    if let Some(dest) = &self.destination_field {
      if dest.trim().is_empty() {
        return Err("Destination field cannot be empty when specified".to_string());
      }
    }

    Ok(())
  }

  /// 获取目标字段名称
  pub fn get_destination_field(&self) -> String {
    self.destination_field.clone().unwrap_or_else(|| {
      // 从源字段路径中提取最后的字段名
      self.field_to_split
        .split('.')
        .last()
        .unwrap_or(&self.field_to_split)
        .to_string()
    })
  }
}

/// Split Out 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitOutConfig {
  /// 要拆分的字段列表
  pub fields_to_split: Vec<FieldToSplit>,
  /// 字段包含策略
  pub include_strategy: IncludeStrategy,
  /// 选择性包含的字段列表
  pub fields_to_include: Vec<String>,
  /// 是否禁用点记号
  pub disable_dot_notation: bool,
  /// 是否包含二进制数据
  pub include_binary: bool,
}

impl Default for SplitOutConfig {
  fn default() -> Self {
    Self {
      fields_to_split: vec![],
      include_strategy: IncludeStrategy::NoOtherFields,
      fields_to_include: vec![],
      disable_dot_notation: false,
      include_binary: false,
    }
  }
}

impl SplitOutConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.fields_to_split.is_empty() {
      return Err("At least one field to split must be specified".to_string());
    }

    // 验证所有拆分字段配置
    for field in &self.fields_to_split {
      field.validate()?;
    }

    // 验证选择性包含策略的字段列表
    if self.include_strategy == IncludeStrategy::SelectedOtherFields && self.fields_to_include.is_empty() {
      return Err("Fields to include must be specified when using selectedOtherFields strategy".to_string());
    }

    Ok(())
  }
}

/// Split Out 节点
///
/// 将输入项中的数组或对象字段拆分为多个独立的输出项。
/// 这是数据处理工作流中的核心节点，主要用于：
/// - API 响应数据的扁平化处理
/// - 复杂数据结构的拆分
/// - 批量数据的准备阶段
/// - 数据清洗和格式转换
pub struct SplitOutNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl SplitOutNode {
  /// 创建新的 Split Out 节点实例
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(SplitOutV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  /// 创建基础节点定义构建器
  fn base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(NodeKind::from(SPLIT_OUT_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform])
      .display_name("Split Out")
      .description("Turn a list inside item(s) into separate items")
      .icon("file-split")
      .version(Version::new(1, 0, 0));
    base
  }
}

impl Node for SplitOutNode {
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
    let node = SplitOutNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::SplitOut");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform]);
    assert_eq!(&definition.display_name, "Split Out");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = SplitOutNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_field_to_split_validation() {
    // 有效的字段配置
    let valid_field = FieldToSplit {
      field_to_split: "data.items".to_string(),
      destination_field: Some("items".to_string()),
    };
    assert!(valid_field.validate().is_ok());

    // 空字段名
    let invalid_field = FieldToSplit {
      field_to_split: "".to_string(),
      destination_field: None,
    };
    assert!(invalid_field.validate().is_err());

    // 空目标字段
    let invalid_dest = FieldToSplit {
      field_to_split: "data".to_string(),
      destination_field: Some("".to_string()),
    };
    assert!(invalid_dest.validate().is_err());

    // 获取目标字段名
    let field_with_dest = FieldToSplit {
      field_to_split: "data.items".to_string(),
      destination_field: Some("processed_items".to_string()),
    };
    assert_eq!(field_with_dest.get_destination_field(), "processed_items");

    let field_without_dest = FieldToSplit {
      field_to_split: "data.items".to_string(),
      destination_field: None,
    };
    assert_eq!(field_without_dest.get_destination_field(), "items");
  }

  #[test]
  fn test_split_out_config_validation() {
    // 有效的配置
    let valid_config = SplitOutConfig {
      fields_to_split: vec![FieldToSplit {
        field_to_split: "items".to_string(),
        destination_field: None,
      }],
      include_strategy: IncludeStrategy::AllOtherFields,
      fields_to_include: vec![],
      disable_dot_notation: false,
      include_binary: false,
    };
    assert!(valid_config.validate().is_ok());

    // 无效配置：没有拆分字段
    let invalid_config = SplitOutConfig {
      fields_to_split: vec![],
      include_strategy: IncludeStrategy::NoOtherFields,
      fields_to_include: vec![],
      disable_dot_notation: false,
      include_binary: false,
    };
    assert!(invalid_config.validate().is_err());

    // 无效配置：选择性包含但没有字段列表
    let invalid_selected = SplitOutConfig {
      fields_to_split: vec![FieldToSplit {
        field_to_split: "items".to_string(),
        destination_field: None,
      }],
      include_strategy: IncludeStrategy::SelectedOtherFields,
      fields_to_include: vec![],
      disable_dot_notation: false,
      include_binary: false,
    };
    assert!(invalid_selected.validate().is_err());
  }

  #[test]
  fn test_include_strategy_serialization() {
    let strategy = IncludeStrategy::NoOtherFields;
    let serialized = serde_json::to_string(&strategy).unwrap();
    let deserialized: IncludeStrategy = serde_json::from_str(&serialized).unwrap();
    assert_eq!(strategy, deserialized);

    let strategy = IncludeStrategy::AllOtherFields;
    let serialized = serde_json::to_string(&strategy).unwrap();
    let deserialized: IncludeStrategy = serde_json::from_str(&serialized).unwrap();
    assert_eq!(strategy, deserialized);

    let strategy = IncludeStrategy::SelectedOtherFields;
    let serialized = serde_json::to_string(&strategy).unwrap();
    let deserialized: IncludeStrategy = serde_json::from_str(&serialized).unwrap();
    assert_eq!(strategy, deserialized);
  }

  #[test]
  fn test_include_strategy_display() {
    assert_eq!(IncludeStrategy::NoOtherFields.to_string(), "noOtherFields");
    assert_eq!(IncludeStrategy::AllOtherFields.to_string(), "allOtherFields");
    assert_eq!(IncludeStrategy::SelectedOtherFields.to_string(), "selectedOtherFields");
  }

  #[test]
  fn test_split_out_config_default() {
    let config = SplitOutConfig::default();
    assert!(config.fields_to_split.is_empty());
    assert_eq!(config.include_strategy, IncludeStrategy::NoOtherFields);
    assert!(config.fields_to_include.is_empty());
    assert!(!config.disable_dot_notation);
    assert!(!config.include_binary);
  }
}