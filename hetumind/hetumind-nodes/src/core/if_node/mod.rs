//! If 条件判断节点实现
//!
//! 参考 n8n 的 If 节点设计，用于根据条件判断分割工作流执行路径。
//! 支持多种数据类型的比较操作，包括字符串、数字、布尔值、日期时间等。

use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use hetumind_core::{
  types::DataType,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, NodeDefinition, NodeExecutionContext, NodeExecutionError,
    NodeExecutor, NodeGroupKind, NodeKind, ValidationError, make_execution_data_map,
  },
};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod utils;
mod v1;

use v1::IfNodeV1;

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

#[derive(Debug)]
pub struct IfNode(IfNodeV1);

impl Default for IfNode {
  fn default() -> Self {
    let base = NodeDefinition::builder()
      .kind(NodeKind::from(IF_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform])
      .display_name("If")
      .description("根据条件判断分割工作流执行路径。支持多种数据类型的比较操作。")
      .icon("code-branch")
      .versions([1])
      .build();
    Self(IfNodeV1::new(base))
  }
}

impl Deref for IfNode {
  type Target = IfNodeV1;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// If 条件判断节点
///
/// 用于根据条件判断将工作流分为 true 和 false 两个分支。
/// 支持多种数据类型的比较操作，以及 AND/OR 逻辑组合。
///
/// # 输出分支
/// - `true`: 条件满足时的输出分支
/// - `false`: 条件不满足时的输出分支
///
/// # 支持的数据类型
/// - String: 字符串比较
/// - Number: 数值比较
/// - Boolean: 布尔值比较
/// - DateTime: 日期时间比较
///
/// # 支持的比较操作
/// - equal: 等于
/// - notEqual: 不等于
/// - contains: 包含
/// - notContains: 不包含
/// - startsWith: 以...开始
/// - endsWith: 以...结束
/// - regex: 正则表达式匹配
/// - isEmpty: 为空
/// - isNotEmpty: 不为空
/// - greaterThan: 大于
/// - lessThan: 小于
/// - greaterThanOrEqual: 大于等于
/// - lessThanOrEqual: 小于等于
#[async_trait]
impl NodeExecutor for IfNode {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.0.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    info!(
      "开始执行 If 条件判断节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      input_data
    } else {
      warn!("If 节点没有接收到输入数据");
      // 如果没有输入数据，默认走 false 分支
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_null(), ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取条件配置
    let conditions: Vec<ConditionConfig> = node.get_parameter("conditions")?;
    let logic_combination: LogicCombination =
      node.get_optional_parameter("combination").unwrap_or(LogicCombination::And);

    debug!("条件判断: {} 个条件，逻辑组合: {:?}", conditions.len(), logic_combination);

    // 最终结果
    let mut logic_value = true;

    for (index, input) in input_items.iter().enumerate() {
      let result = self.evaluate_conditions(&conditions, &logic_combination, input.json())?;
      logic_value = logic_value && result;
      debug!("输入数据项:{} 结果:{} 条件判断结果:{}", index, result, logic_value);
    }

    let res = if logic_value {
      vec![ExecutionDataItems::new_items(input_items), ExecutionDataItems::new_null()]
    } else {
      vec![ExecutionDataItems::new_null(), ExecutionDataItems::new_items(input_items)]
    };

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, res)]))
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = IfNode::default();
    let metadata = node.definition();

    assert_eq!(metadata.kind.as_ref(), "If");
    assert_eq!(&metadata.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&metadata.display_name, "If");
    assert_eq!(metadata.inputs.len(), 1);
    assert_eq!(metadata.outputs.len(), 2);
  }

  #[test]
  fn test_node_ports() {
    let node = IfNode::default();

    let input_ports = &node.definition().inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &node.definition().outputs[..];
    assert_eq!(output_ports.len(), 2);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
    assert_eq!(output_ports[1].kind, ConnectionKind::Main);
  }
}
