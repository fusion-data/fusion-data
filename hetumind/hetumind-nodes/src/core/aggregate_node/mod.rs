//! Aggregate 数据聚合节点实现
//!
//! 参考 n8n 的 Aggregate 节点设计，用于将多个数据项的字段合并成单个数据项中的列表。
//! 支持两种聚合模式：Individual Fields 和 All Item Data。

use std::sync::Arc;

use fusion_common::ahash::HashSet;
use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeDefinition, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod aggregate_v1;
mod utils;

use aggregate_v1::AggregateV1;

use crate::constants::AGGREGATE_NODE_KIND;

/// 聚合模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateMode {
  /// 按字段聚合：选择性聚合指定字段
  AggregateIndividualFields,
  /// 全部数据聚合：将所有数据项聚合成单个列表
  AggregateAllItemData,
}

/// 字段聚合配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldToAggregate {
  /// 输入字段名称
  pub field_to_aggregate: String,
  /// 是否重命名字段
  pub rename_field: bool,
  /// 输出字段名称
  pub output_field_name: Option<String>,
}

impl FieldToAggregate {
  /// 验证字段配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.field_to_aggregate.trim().is_empty() {
      return Err("Input field name cannot be empty".to_string());
    }

    if self.rename_field {
      if let Some(output_name) = &self.output_field_name {
        if output_name.trim().is_empty() {
          return Err("Output field name cannot be empty when rename is enabled".to_string());
        }
      } else {
        return Err("Output field name is required when rename is enabled".to_string());
      }
    }

    Ok(())
  }

  /// 获取输出字段名称
  pub fn get_output_field_name(&self) -> String {
    if self.rename_field {
      self.output_field_name.clone().unwrap_or_else(|| self.field_to_aggregate.clone())
    } else {
      self.field_to_aggregate.clone()
    }
  }
}

/// 聚合配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateOptions {
  /// 是否禁用点记号
  pub disable_dot_notation: bool,
  /// 是否合并列表
  pub merge_lists: bool,
  /// 是否保留缺失值
  pub keep_missing: bool,
  /// 是否包含二进制数据
  pub include_binaries: bool,
  /// 是否只保留唯一的二进制数据
  pub keep_only_unique: bool,
}

impl Default for AggregateOptions {
  fn default() -> Self {
    Self {
      disable_dot_notation: false,
      merge_lists: false,
      keep_missing: true,
      include_binaries: false,
      keep_only_unique: false,
    }
  }
}

/// 聚合节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateConfig {
  /// 聚合模式
  pub aggregate: AggregateMode,
  /// 要聚合的字段列表（Individual Fields 模式使用）
  pub fields_to_aggregate: Vec<FieldToAggregate>,
  /// 目标字段名称（All Item Data 模式使用）
  pub destination_field_name: Option<String>,
  /// 要排除的字段列表（All Item Data 模式使用）
  pub fields_to_exclude: Vec<String>,
  /// 要包含的字段列表（All Item Data 模式使用）
  pub fields_to_include: Vec<String>,
  /// 聚合选项
  pub options: AggregateOptions,
}

impl AggregateConfig {
  /// 验证聚合配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    match self.aggregate {
      AggregateMode::AggregateIndividualFields => {
        if self.fields_to_aggregate.is_empty() {
          return Err("At least one field must be specified for Individual Fields aggregation".to_string());
        }

        // 验证输出字段名称唯一性
        let mut output_field_names = HashSet::default();
        for field in &self.fields_to_aggregate {
          field.validate()?;
          let output_name = field.get_output_field_name();
          if output_field_names.contains(&output_name) {
            return Err(format!("The '{}' output field is used more than once", output_name));
          }
          output_field_names.insert(output_name);
        }
      }
      AggregateMode::AggregateAllItemData => {
        if let Some(destination) = &self.destination_field_name {
          if destination.trim().is_empty() {
            return Err("Destination field name cannot be empty".to_string());
          }
        } else {
          return Err("Destination field name is required for All Item Data aggregation".to_string());
        }
      }
    }

    Ok(())
  }
}

pub struct AggregateNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl AggregateNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(AggregateV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(AGGREGATE_NODE_KIND, "Aggregate")
      .add_group(NodeGroupKind::Transform)
      .with_description("Combine a field from many items into a list in a single item")
      .with_icon("object-group")
  }
}

impl Node for AggregateNode {
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
    let node = AggregateNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::Aggregate");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform]);
    assert_eq!(&definition.display_name, "Aggregate");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = AggregateNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_field_to_aggregate_validation() {
    // 有效的字段配置
    let valid_field =
      FieldToAggregate { field_to_aggregate: "name".to_string(), rename_field: false, output_field_name: None };
    assert!(valid_field.validate().is_ok());

    // 无效的空字段名
    let invalid_field =
      FieldToAggregate { field_to_aggregate: "".to_string(), rename_field: false, output_field_name: None };
    assert!(invalid_field.validate().is_err());

    // 重命名但缺少输出字段名
    let invalid_rename =
      FieldToAggregate { field_to_aggregate: "name".to_string(), rename_field: true, output_field_name: None };
    assert!(invalid_rename.validate().is_err());

    // 有效的重命名配置
    let valid_rename = FieldToAggregate {
      field_to_aggregate: "name".to_string(),
      rename_field: true,
      output_field_name: Some("full_name".to_string()),
    };
    assert!(valid_rename.validate().is_ok());
  }

  #[test]
  fn test_aggregate_config_validation() {
    // 有效的 Individual Fields 配置
    let valid_individual_fields = AggregateConfig {
      aggregate: AggregateMode::AggregateIndividualFields,
      fields_to_aggregate: vec![FieldToAggregate {
        field_to_aggregate: "name".to_string(),
        rename_field: false,
        output_field_name: None,
      }],
      destination_field_name: None,
      fields_to_exclude: vec![],
      fields_to_include: vec![],
      options: AggregateOptions::default(),
    };
    assert!(valid_individual_fields.validate().is_ok());

    // 无效的 Individual Fields 配置（没有字段）
    let invalid_individual_fields = AggregateConfig {
      aggregate: AggregateMode::AggregateIndividualFields,
      fields_to_aggregate: vec![],
      destination_field_name: None,
      fields_to_exclude: vec![],
      fields_to_include: vec![],
      options: AggregateOptions::default(),
    };
    assert!(invalid_individual_fields.validate().is_err());

    // 有效的 All Item Data 配置
    let valid_all_item_data = AggregateConfig {
      aggregate: AggregateMode::AggregateAllItemData,
      fields_to_aggregate: vec![],
      destination_field_name: Some("items".to_string()),
      fields_to_exclude: vec![],
      fields_to_include: vec![],
      options: AggregateOptions::default(),
    };
    assert!(valid_all_item_data.validate().is_ok());

    // 无效的 All Item Data 配置（缺少目标字段名）
    let invalid_all_item_data = AggregateConfig {
      aggregate: AggregateMode::AggregateAllItemData,
      fields_to_aggregate: vec![],
      destination_field_name: None,
      fields_to_exclude: vec![],
      fields_to_include: vec![],
      options: AggregateOptions::default(),
    };
    assert!(invalid_all_item_data.validate().is_err());
  }

  #[test]
  fn test_aggregate_mode_serialization() {
    let mode = AggregateMode::AggregateIndividualFields;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: AggregateMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);

    let mode = AggregateMode::AggregateAllItemData;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: AggregateMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);
  }
}
