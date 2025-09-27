//! Set 数据设置节点实现
//!
//! 参考 n8n 的 Set 节点设计，用于设置、修改或删除数据字段。
//! 支持多种操作类型和数据源，是数据处理工作流中的重要节点。

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, InputPortConfig, NodeDefinition, NodeExecutionContext, NodeExecutionError,
  NodeExecutor, NodeGroupKind, NodeKind, NodeProperties, NodePropertyKind, OutputPortConfig, WorkflowNode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use log::{debug, info, warn};

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
  /// 设置字段值
  Set,
  /// 删除字段
  Remove,
  /// 从其他字段复制值
  Copy,
  /// 数值增加
  Increment,
  /// 数组追加元素
  Append,
}

/// 数据来源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueSourceKind {
  /// 静态值
  Static,
  /// 表达式（JSON Path）
  Expression,
  /// 当前时间戳
  CurrentTimestamp,
  /// 随机值
  Random,
}

/// 设置操作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetOperation {
  /// 目标字段路径
  pub field_path: String,
  /// 操作类型
  pub kind: OperationKind,
  /// 值来源类型
  pub value_source: ValueSourceKind,
  /// 设置的值（当 value_source 为 StaticValue 或 Expression 时使用）
  pub value: Option<Value>,
  /// 是否保留原始类型
  pub keep: Option<bool>,
}

/// Set 数据设置节点
///
/// 用于对输入数据进行字段设置、修改或删除操作。
/// 支持多种数据来源和操作类型，可以构建复杂的数据转换逻辑。
///
/// # 操作类型
/// - `Set`: 设置字段值
/// - `Remove`: 删除字段
/// - `Copy`: 从其他字段复制值
/// - `Increment`: 数值增加
/// - `Append`: 数组追加元素
///
/// # 数据来源
/// - `StaticValue`: 静态值
/// - `Expression`: 表达式（如 $.field.subfield）
/// - `CurrentTimestamp`: 当前时间戳
/// - `RandomValue`: 随机值
///
/// # 输入/输出
/// - 输入：任意 JSON 数据
/// - 输出：修改后的 JSON 数据
#[derive(Debug, Clone)]
pub struct SetNode {
  definition: Arc<NodeDefinition>,
}

impl Default for SetNode {
  fn default() -> Self {
    Self { definition: Arc::new(create_definition()) }
  }
}

#[async_trait]
impl NodeExecutor for SetNode {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<Vec<Vec<ExecutionData>>, NodeExecutionError> {
    let node = context.current_node()?;
    info!(
      "开始执行 Set 数据设置节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // 获取输入数据
    if context.input_data.is_empty() {
      warn!("Set 节点没有接收到输入数据");
      return Ok(vec![]);
    }

    // 获取操作配置
    let operations = self.get_operations(node)?;

    debug!("Set 操作配置: {} 个操作", operations.len());

    let mut results = Vec::new();

    // 处理每个输入数据项
    for (index, input_data) in context.input_data.iter().enumerate() {
      let mut modified_data = input_data.json.clone();

      // 执行所有设置操作
      for operation in &operations {
        modified_data = self.apply_operation(&modified_data, operation)?;
      }

      results.push(ExecutionData {
        json: modified_data,
        source: Some(DataSource {
          node_name: context.current_node_name.clone(),
          output_port: "main".to_string(),
          output_index: index,
        }),
        index: input_data.index,
        binary: input_data.binary.clone(),
      });
    }

    info!("Set 节点执行完成: 处理 {} 项数据", results.len());

    Ok(vec![results])
  }
}

impl SetNode {
  /// 从节点参数中获取操作参数
  fn get_operations(&self, node: &WorkflowNode) -> Result<Vec<SetOperation>, NodeExecutionError> {
    let operations: Vec<SetOperation> = node.get_parameter("operations")?;
    Ok(operations)
  }

  /// 应用单个设置操作
  fn apply_operation(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    match operation.kind {
      OperationKind::Set => self.apply_set_operation(data, operation),
      OperationKind::Remove => self.apply_remove_operation(data, operation),
      OperationKind::Copy => self.apply_copy_operation(data, operation),
      OperationKind::Increment => self.apply_increment_operation(data, operation),
      OperationKind::Append => self.apply_append_operation(data, operation),
    }
  }

  /// 应用设置操作
  fn apply_set_operation(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    let value_to_set = self.resolve_value(data, operation)?;
    self.set_nested_value(data, &operation.field_path, value_to_set)
  }

  /// 应用删除操作
  fn apply_remove_operation(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    self.remove_nested_value(data, &operation.field_path)
  }

  /// 应用复制操作
  fn apply_copy_operation(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    if let Some(source_path) = operation.value.as_ref().and_then(|v| v.as_str()) {
      if let Some(source_value) = self.get_nested_value(data, source_path) {
        return self.set_nested_value(data, &operation.field_path, source_value);
      }
    }

    // 如果源路径不存在，保持原数据不变
    Ok(data.clone())
  }

  /// 应用增加操作（仅适用于数值）
  fn apply_increment_operation(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    let increment_value = self.resolve_value(data, operation)?;
    let increment_num = self.to_number(&increment_value).unwrap_or(1.0);

    if let Some(current_value) = self.get_nested_value(data, &operation.field_path) {
      if let Some(current_num) = self.to_number(&current_value) {
        let new_value = json!(current_num + increment_num);
        return self.set_nested_value(data, &operation.field_path, new_value);
      }
    }

    // 如果字段不存在或不是数值，设置为增量值
    self.set_nested_value(data, &operation.field_path, increment_value)
  }

  /// 应用追加操作（仅适用于数组）
  fn apply_append_operation(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    let value_to_append = self.resolve_value(data, operation)?;

    if let Some(current_value) = self.get_nested_value(data, &operation.field_path) {
      if let Some(current_array) = current_value.as_array() {
        let mut new_array = current_array.clone();
        new_array.push(value_to_append);
        return self.set_nested_value(data, &operation.field_path, json!(new_array));
      }
    }

    // 如果字段不存在或不是数组，创建新数组
    self.set_nested_value(data, &operation.field_path, json!([value_to_append]))
  }

  /// 解析操作值
  fn resolve_value(&self, data: &Value, operation: &SetOperation) -> Result<Value, NodeExecutionError> {
    match operation.value_source {
      ValueSourceKind::Static => Ok(operation.value.clone().unwrap_or(Value::Null)),
      ValueSourceKind::Expression => {
        if let Some(expr) = operation.value.as_ref().and_then(|v| v.as_str()) {
          if let Some(path) = expr.strip_prefix("$.") {
            // 简单的 JSON Path 支持
            return Ok(self.get_nested_value(data, path).unwrap_or(Value::Null));
          }
        }
        Ok(operation.value.clone().unwrap_or(Value::Null))
      }
      ValueSourceKind::CurrentTimestamp => {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        Ok(json!(timestamp))
      }
      ValueSourceKind::Random => {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let random_num = hasher.finish() as f64 / u64::MAX as f64;
        Ok(json!(random_num))
      }
    }
  }

  /// 获取嵌套值
  fn get_nested_value(&self, data: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;

    for part in parts {
      match current {
        Value::Object(obj) => {
          current = obj.get(part)?;
        }
        _ => return None,
      }
    }

    Some(current.clone())
  }

  /// 设置嵌套值
  fn set_nested_value(&self, data: &Value, path: &str, value: Value) -> Result<Value, NodeExecutionError> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
      return Ok(value);
    }

    let mut result = data.clone();

    // 确保根对象是一个对象
    if !result.is_object() {
      result = json!({});
    }

    Self::set_nested_value_recursive(&mut result, &parts, 0, value)?;
    Ok(result)
  }

  /// 递归设置嵌套值
  fn set_nested_value_recursive(
    current: &mut Value,
    parts: &[&str],
    index: usize,
    value: Value,
  ) -> Result<(), NodeExecutionError> {
    if index >= parts.len() {
      return Ok(());
    }

    let part = parts[index];

    // 确保当前值是一个对象
    if !current.is_object() {
      *current = json!({});
    }

    if index == parts.len() - 1 {
      // 最后一级，直接设置值
      if let Some(obj) = current.as_object_mut() {
        obj.insert(part.to_string(), value);
      }
    } else {
      // 中间层级，递归处理
      if let Some(obj) = current.as_object_mut() {
        if !obj.contains_key(part) {
          obj.insert(part.to_string(), json!({}));
        }
        if let Some(next_value) = obj.get_mut(part) {
          Self::set_nested_value_recursive(next_value, parts, index + 1, value)?;
        }
      }
    }

    Ok(())
  }

  /// 删除嵌套值
  fn remove_nested_value(&self, data: &Value, path: &str) -> Result<Value, NodeExecutionError> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
      return Ok(data.clone());
    }

    let mut result = data.clone();
    self.remove_nested_value_recursive(&mut result, &parts, 0)?;
    Ok(result)
  }

  /// 递归删除嵌套值
  fn remove_nested_value_recursive(
    &self,
    current: &mut Value,
    parts: &[&str],
    index: usize,
  ) -> Result<(), NodeExecutionError> {
    if index >= parts.len() {
      return Ok(());
    }

    let part = parts[index];

    if let Some(obj) = current.as_object_mut() {
      if index == parts.len() - 1 {
        // 最后一级，删除字段
        obj.remove(part);
      } else if let Some(next_value) = obj.get_mut(part) {
        // 中间层级，递归处理
        self.remove_nested_value_recursive(next_value, parts, index + 1)?;
      }
    }

    Ok(())
  }

  /// 转换为数值
  fn to_number(&self, value: &Value) -> Option<f64> {
    match value {
      Value::Number(n) => n.as_f64(),
      Value::String(s) => s.parse().ok(),
      _ => None,
    }
  }

  pub const NODE_KIND: &str = "hetumind_nodes::core::Set";
}

/// 创建节点元数据
fn create_definition() -> NodeDefinition {
  NodeDefinition::builder()
    .kind(NodeKind::from(SetNode::NODE_KIND))
    .versions(vec![1])
    .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output])
    .display_name("Set")
    .description("设置、修改或删除数据字段。支持多种操作类型和数据来源。")
    .icon("edit")
    .inputs(vec![InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
    .outputs(vec![OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
    .properties(vec![
      NodeProperties::builder()
        .name("operations")
        .kind(NodePropertyKind::Collection)
        .required(true)
        .display_name("操作列表")
        .description("要执行的设置操作列表")
        .value(json!([]))
        .placeholder("添加操作...")
        .build(),
      NodeProperties::builder()
        .name("keep_original_type")
        .kind(NodePropertyKind::Boolean)
        .required(false)
        .display_name("保持原始类型")
        .description("是否尝试保持字段的原始数据类型")
        .value(json!(false))
        .build(),
    ])
    .build()
}
