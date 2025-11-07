//! If 条件判断节点实现
//!
//! 参考 n8n 的 If 节点设计，用于根据条件判断分割工作流执行路径。
//! 支持多种数据类型的比较操作，包括字符串、数字、布尔值、日期时间等。

use std::sync::Arc;

use hetumind_core::{
  types::{DataType, JsonValue},
  version::Version,
  workflow::{
    FlowNodeRef, Node, NodeDescription, NodeExecutionError, NodeGroupKind, NodeType, RegistrationError, ValidationError,
  },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod if_v1;
pub mod utils;

#[cfg(test)]
mod json_path_test;

use if_v1::IfV1;

use crate::constants::IF_NODE_KIND;

/// 条件配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionConfig {
  /// 左值（支持表达式）
  pub left: Value,
  /// 比较操作
  pub op: ComparisonOperation,
  /// 右值（比较目标，支持表达式）
  pub right: Option<Value>,
  /// 数据类型
  pub data_type: DataType,
  /// 条件描述（可选，用于调试）
  pub description: Option<String>,
  /// 是否启用此条件
  pub enabled: Option<bool>,
  /// 自定义函数调用（高级功能）
  pub custom_function: Option<CustomFunctionCall>,
}

/// 自定义函数调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFunctionCall {
  /// 函数名
  pub name: String,
  /// 函数参数
  pub parameters: Vec<Value>,
  /// 函数类型
  pub function_type: FunctionType,
}

/// 函数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionType {
  /// 字符串函数
  String,
  /// 数值函数
  Number,
  /// 日期函数
  Date,
  /// 逻辑函数
  Logic,
  /// 自定义函数
  Custom,
}

/// 高级条件组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConditionGroup {
  /// 条件组合名称
  pub name: Option<String>,
  /// 条件组合逻辑
  pub groups: Vec<ConditionGroup>,
  /// 组合逻辑（AND/OR）
  pub combine_operation: LogicCombination,
}

/// 条件组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionGroup {
  /// 组内条件
  pub conditions: Vec<ConditionConfig>,
  /// 组内逻辑组合
  pub combine_operation: LogicCombination,
  /// 组权重（用于加权评估）
  pub weight: Option<f64>,
}

impl ConditionConfig {
  /// Verify if the condition configuration is valid.
  #[allow(dead_code)]
  pub fn validate(&self) -> Result<(), ValidationError> {
    // 非[相等或不相等]时，右值不可为空
    if !(self.op == ComparisonOperation::Eq || self.op == ComparisonOperation::Nq) && self.right.is_none() {
      return Err(ValidationError::invalid_field_value("right".to_string(), "Cannot be empty".to_string()));
    }

    // 验证自定义函数调用
    if let Some(custom_func) = &self.custom_function
      && custom_func.name.is_empty()
    {
      return Err(ValidationError::invalid_field_value(
        "custom_function.name".to_string(),
        "Function name cannot be empty".to_string(),
      ));
    }

    Ok(())
  }

  /// 检查条件是否启用
  pub fn is_enabled(&self) -> bool {
    self.enabled.unwrap_or(true)
  }

  /// 获取条件描述
  #[allow(dead_code)]
  pub fn get_description(&self) -> String {
    self.description.clone().unwrap_or_else(|| {
      format!(
        "{} {} {}",
        format_value(&self.left),
        format_operation(&self.op),
        self.right.as_ref().map(format_value).unwrap_or("null".to_string())
      )
    })
  }
}

impl AdvancedConditionGroup {
  /// 评估高级条件组合
  pub fn evaluate(&self, input_data: &JsonValue, options: &IfNodeOptions) -> Result<bool, NodeExecutionError> {
    if self.groups.is_empty() {
      return Ok(false);
    }

    let mut group_results = Vec::new();

    for group in &self.groups {
      let group_result = if group.conditions.is_empty() {
        false
      } else {
        // 评估组内条件
        let mut condition_results = Vec::new();

        for condition in &group.conditions {
          if !condition.is_enabled() {
            continue;
          }

          let result =
            crate::core::if_node::utils::evaluate_single_condition_with_options(condition, input_data, options)?;

          // 应用权重（如果有的话）
          let weighted_result = if let Some(weight) = group.weight {
            if weight <= 0.0 || weight > 1.0 {
              log::warn!("条件组权重 {} 超出范围 [0,1]，将被忽略", weight);
              result
            } else {
              // 权重影响：这里简化处理，实际可以根据需要调整逻辑
              if result { weight > 0.5 } else { weight < 0.5 }
            }
          } else {
            result
          };

          condition_results.push(weighted_result);
        }

        // 组内条件组合
        match group.combine_operation {
          LogicCombination::And => condition_results.iter().all(|&x| x),
          LogicCombination::Or => condition_results.iter().any(|&x| x),
        }
      };

      group_results.push(group_result);
    }

    // 组间条件组合
    let final_result = match self.combine_operation {
      LogicCombination::And => group_results.iter().all(|&x| x),
      LogicCombination::Or => group_results.iter().any(|&x| x),
    };

    Ok(final_result)
  }
}

/// 格式化值用于显示
fn format_value(value: &Value) -> String {
  match value {
    Value::String(s) => format!("\"{}\"", s),
    Value::Number(n) => n.to_string(),
    Value::Bool(b) => b.to_string(),
    Value::Null => "null".to_string(),
    Value::Array(arr) => format!("[{}]", arr.iter().map(format_value).collect::<Vec<_>>().join(", ")),
    Value::Object(obj) => {
      let pairs: Vec<String> = obj.iter().map(|(k, v)| format!("{}: {}", k, format_value(v))).collect();
      format!("{{{}}}", pairs.join(", "))
    }
  }
}

/// 格式化操作符用于显示
fn format_operation(op: &ComparisonOperation) -> String {
  match op {
    ComparisonOperation::Eq => "==".to_string(),
    ComparisonOperation::Nq => "!=".to_string(),
    ComparisonOperation::Gt => ">".to_string(),
    ComparisonOperation::Lt => "<".to_string(),
    ComparisonOperation::Gte => ">=".to_string(),
    ComparisonOperation::Lte => "<=".to_string(),
    ComparisonOperation::Contains => "contains".to_string(),
    ComparisonOperation::NotContains => "not contains".to_string(),
    ComparisonOperation::StartsWith => "starts with".to_string(),
    ComparisonOperation::EndsWith => "ends with".to_string(),
    ComparisonOperation::Regex => "matches".to_string(),
    ComparisonOperation::Empty => "is empty".to_string(),
    ComparisonOperation::NotEmpty => "is not empty".to_string(),
    ComparisonOperation::Is => "is true".to_string(),
    ComparisonOperation::Not => "is false".to_string(),
  }
}

/// 比较操作类型
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperation {
  /// Equal
  #[default]
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

/// IfNode 配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfNodeOptions {
  /// 类型验证模式
  pub type_validation_mode: TypeValidationMode,
  /// 忽略大小写（字符串比较）
  pub ignore_case: bool,
  /// 遇到错误时是否继续执行
  pub continue_on_error: bool,
  /// 错误处理策略
  pub error_handling_strategy: ErrorHandlingStrategy,
  /// 默认值处理
  pub default_value_handling: DefaultValueHandling,
  /// 调试模式
  pub debug_mode: bool,
}

/// 类型验证模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeValidationMode {
  /// 严格模式：类型不匹配时失败
  Strict,
  /// 宽松模式：尝试自动类型转换
  Loose,
  /// 智能模式：根据上下文自动选择验证策略
  Smart,
}

/// 错误处理策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandlingStrategy {
  /// 停止执行
  StopExecution,
  /// 跳过当前项
  SkipItem,
  /// 走默认分支（false分支）
  GoToDefaultBranch,
  /// 记录错误但继续
  LogAndContinue,
}

/// 默认值处理策略
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultValueHandling {
  /// 使用 null
  UseNull,
  /// 使用空字符串
  UseEmptyString,
  /// 使用默认数值（0）
  UseZero,
  /// 使用默认布尔值（false）
  UseFalse,
  /// 自定义默认值
  Custom(Value),
}

impl Default for IfNodeOptions {
  fn default() -> Self {
    Self {
      type_validation_mode: TypeValidationMode::Loose,
      ignore_case: false,
      continue_on_error: true,
      error_handling_strategy: ErrorHandlingStrategy::GoToDefaultBranch,
      default_value_handling: DefaultValueHandling::UseNull,
      debug_mode: false,
    }
  }
}

pub struct IfNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl IfNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(IfV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.description().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDescription {
    NodeDescription::new(IF_NODE_KIND, "If")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description(
        "Splits workflow execution paths based on conditions. Supports comparison operations for multiple data types.",
      )
      .with_icon("code-branch")
  }
}

impl Node for IfNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn node_type(&self) -> NodeType {
    self.executors[0].description().node_type.clone()
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::{NodeConnectionKind, NodeGroupKind};

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = IfNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().description();

    assert_eq!(definition.node_type.as_ref(), "hetumind_nodes::If");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "If");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 2);
  }

  #[test]
  fn test_node_ports() {
    let node = IfNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().description();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, NodeConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 2);
    assert_eq!(output_ports[0].kind, NodeConnectionKind::Main);
    assert_eq!(output_ports[1].kind, NodeConnectionKind::Main);
  }
}
