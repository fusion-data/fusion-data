//! Merge 数据合并节点实现
//!
//! 参考 n8n 的 Merge 节点设计，用于合并多个分支的数据流。
//! 支持多种合并模式，包括简单追加、按键合并、按索引合并等。

use std::sync::Arc;

use ahash::HashMap;
use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, InputPortConfig, NodeDefinition, NodeExecutionContext, NodeExecutionError,
  NodeExecutable, NodeGroupKind, NodeKind, NodeName, NodeProperties, NodePropertyKind, OutputPortConfig, ValidationError,
  WorkflowNode,
};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Merge 数据合并节点
///
/// 用于合并多个输入分支的数据流，支持多种合并策略。
/// 常用于 If 节点分支后重新合并数据流的场景。
///
/// # 合并模式
/// - `Append`: 简单追加所有输入数据
/// - `MergeByKey`: 按指定字段合并数据
/// - `MergeByIndex`: 按索引位置合并数据
/// - `WaitForAll`: 等待所有输入完成后合并
///
/// # 输入端口
/// - 支持多个输入端口（默认2个，可配置）
///
/// # 输出端口
/// - 单个主输出端口，包含合并后的数据
#[derive(Debug, Clone)]
pub struct MergeNode {
  definition: Arc<NodeDefinition>,
}

/// 合并模式
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for MergeNode {
  fn default() -> Self {
    Self { definition: Arc::new(create_definition()) }
  }
}

#[async_trait]
impl NodeExecutable for MergeNode {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<Vec<Vec<ExecutionData>>, NodeExecutionError> {
    let node = context.current_node()?;
    info!(
      "开始执行 Merge 数据合并节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // 获取输入数据
    if context.input_data.is_empty() {
      warn!("Merge 节点没有接收到输入数据");
      return Ok(vec![]);
    }

    // 获取合并配置
    let config = self.get_merge_config(node)?;

    debug!("合并配置: 模式={:?}, 合并键={:?}, 输入数据量={}", config.mode, config.merge_key, context.input_data.len());

    // 根据合并模式执行不同的合并逻辑
    let merged_data = match config.mode {
      MergeMode::Append => self.merge_append(&context.input_data, context.current_node_name.clone())?,
      MergeMode::MergeByKey => {
        self.merge_by_key(&context.input_data, &config.merge_key, context.current_node_name.clone())?
      }
      MergeMode::MergeByIndex => self.merge_by_index(&context.input_data, context.current_node_name.clone())?,
      MergeMode::WaitForAll => {
        self.merge_wait_for_all(&context.input_data, &config, context.current_node_name.clone())?
      }
    };

    info!("Merge 节点执行完成: 输入 {} 项，输出 {} 项", context.input_data.len(), merged_data.len());

    Ok(vec![merged_data])
  }
}

impl MergeNode {
  /// 从节点参数中获取合并配置
  fn get_merge_config(&self, node: &WorkflowNode) -> Result<MergeConfig, ValidationError> {
    let mode = node.parameters.get_parameter::<MergeMode>("mode")?;
    let merge_key = node.parameters.get_optional_parameter::<String>("merge_key");
    let input_ports = node.parameters.get_optional_parameter::<usize>("input_ports");

    Ok(MergeConfig { mode, merge_key, input_ports })
  }

  /// 简单追加合并
  fn merge_append(
    &self,
    input_data: &[ExecutionData],
    current_node_name: NodeName,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let mut result = Vec::new();

    for (index, data) in input_data.iter().enumerate() {
      result.push(ExecutionData {
        json: data.json.clone(),
        source: Some(DataSource {
          node_name: current_node_name.clone(),
          output_port: "main".to_string(),
          output_index: index,
        }),
        index: data.index,
        binary: data.binary.clone(),
      });
    }

    Ok(result)
  }

  /// 按键合并
  fn merge_by_key(
    &self,
    input_data: &[ExecutionData],
    merge_key: &Option<String>,
    current_node_id: NodeName,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let key_field = merge_key.as_ref().ok_or_else(|| {
      NodeExecutionError::ParameterValidation(ValidationError::RequiredFieldMissing { field: "mergeKey".to_string() })
    })?;

    let mut grouped_data: HashMap<String, Vec<&ExecutionData>> = HashMap::default();

    // 按键分组数据
    for data in input_data {
      let key_value = self.extract_key_value(&data.json, key_field);
      grouped_data.entry(key_value).or_default().push(data);
    }

    let mut result = Vec::new();
    let mut output_index = 0;

    // 合并每个分组的数据
    for (_key, group) in grouped_data {
      if group.is_empty() {
        continue;
      }

      // 合并同一键的所有数据
      let merged_json = self.merge_json_objects(group.iter().map(|d| &d.json).collect())?;

      result.push(ExecutionData {
        json: merged_json,
        source: Some(DataSource { node_name: current_node_id.clone(), output_port: "main".to_string(), output_index }),
        index: output_index,
        binary: group.first().and_then(|d| d.binary.clone()),
      });

      output_index += 1;
    }

    Ok(result)
  }

  /// 按索引合并
  fn merge_by_index(
    &self,
    input_data: &[ExecutionData],
    current_node_id: NodeName,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let mut grouped_data: HashMap<usize, Vec<&ExecutionData>> = HashMap::default();

    // 按索引分组数据
    for data in input_data {
      grouped_data.entry(data.index).or_default().push(data);
    }

    let mut result = Vec::new();
    let mut output_index = 0;

    // 按索引顺序处理
    let mut indices: Vec<_> = grouped_data.keys().copied().collect();
    indices.sort();

    for index in indices {
      if let Some(group) = grouped_data.get(&index) {
        if group.is_empty() {
          continue;
        }

        // 合并同一索引的所有数据
        let merged_json = self.merge_json_objects(group.iter().map(|d| &d.json).collect())?;

        result.push(ExecutionData {
          json: merged_json,
          source: Some(DataSource {
            node_name: current_node_id.clone(),
            output_port: "main".to_string(),
            output_index,
          }),
          index: output_index,
          binary: group.first().and_then(|d| d.binary.clone()),
        });

        output_index += 1;
      }
    }

    Ok(result)
  }

  /// 等待全部输入完成后合并
  fn merge_wait_for_all(
    &self,
    input_data: &[ExecutionData],
    config: &MergeConfig,
    current_node_id: NodeName,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let expected_ports = config.input_ports.unwrap_or(2);

    // 检查是否有足够的输入分支数据
    let mut source_ports = std::collections::HashSet::new();
    for data in input_data {
      if let Some(source) = &data.source {
        source_ports.insert(&source.output_port);
      }
    }

    if source_ports.len() < expected_ports {
      warn!("WaitForAll 模式: 期望 {} 个输入分支，实际收到 {} 个", expected_ports, source_ports.len());
    }

    // 使用简单追加模式合并所有数据
    self.merge_append(input_data, current_node_id)
  }

  /// 提取键值
  fn extract_key_value(&self, data: &Value, key_field: &str) -> String {
    match data.get(key_field) {
      Some(Value::String(s)) => s.clone(),
      Some(value) => value.to_string(),
      None => "null".to_string(),
    }
  }

  /// 合并多个 JSON 对象
  fn merge_json_objects(&self, objects: Vec<&Value>) -> Result<Value, NodeExecutionError> {
    if objects.is_empty() {
      return Ok(Value::Null);
    }

    if objects.len() == 1 {
      return Ok(objects[0].clone());
    }

    let mut merged = Map::new();

    for obj in objects {
      match obj {
        Value::Object(map) => {
          for (key, value) in map {
            // 如果键已存在，尝试合并值
            if let Some(existing_value) = merged.get(key) {
              merged.insert(key.clone(), self.merge_values(existing_value, value)?);
            } else {
              merged.insert(key.clone(), value.clone());
            }
          }
        }
        _ => {
          // 非对象类型，使用 "_value" 作为键
          merged.insert("_value".to_string(), obj.clone());
        }
      }
    }

    Ok(Value::Object(merged))
  }

  /// 合并两个值
  fn merge_values(&self, value1: &Value, value2: &Value) -> Result<Value, NodeExecutionError> {
    match (value1, value2) {
      // 如果都是数组，合并数组
      (Value::Array(arr1), Value::Array(arr2)) => {
        let mut merged = arr1.clone();
        merged.extend(arr2.clone());
        Ok(Value::Array(merged))
      }
      // 如果都是对象，递归合并
      (Value::Object(_), Value::Object(_)) => self.merge_json_objects(vec![value1, value2]),
      // 其他情况，value2 覆盖 value1
      _ => Ok(value2.clone()),
    }
  }

  pub const NODE_KIND: &str = "hetumind_nodes::core::Merge";
}

/// 创建节点元数据
fn create_definition() -> NodeDefinition {
  NodeDefinition::builder()
    .kind(NodeKind::from(MergeNode::NODE_KIND))
    .versions(vec![1])
    .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output])
    .display_name("Merge")
    .description("合并多个分支的数据流。支持多种合并策略。")
    .icon("git-merge".to_string())
    .inputs(vec![
      InputPortConfig::builder()
        .kind(ConnectionKind::Main)
        .display_name("Input")
        .max_connections(128)
        .build(),
    ])
    .outputs(vec![OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
    .properties(vec![
      NodeProperties::builder()
        .name("mode".to_string())
        .kind(NodePropertyKind::Options)
        .required(true)
        .display_name("合并模式")
        .description("数据合并策略")
        .value(json!(MergeMode::Append))
        .options(vec![
          Box::new(
            NodeProperties::builder()
              .display_name("Append")
              .name("append")
              .value(json!(MergeMode::Append))
              .build(),
          ),
          Box::new(
            NodeProperties::builder()
              .display_name("MergeByKey")
              .name("merge_by_key")
              .value(json!(MergeMode::MergeByKey))
              .build(),
          ),
          Box::new(
            NodeProperties::builder()
              .display_name("MergeByIndex")
              .name("merge_by_index")
              .value(json!(MergeMode::MergeByIndex))
              .build(),
          ),
          Box::new(
            NodeProperties::builder()
              .display_name("WaitForAll")
              .name("wait_for_all")
              .value(json!(MergeMode::WaitForAll))
              .build(),
          ),
        ])
        .build(),
      NodeProperties::builder()
        .name("merge_key")
        .kind(NodePropertyKind::String)
        .required(false)
        .display_name("合并键")
        .description("用于按键合并的字段名（仅 mergeByKey 模式）")
        .placeholder("id".to_string())
        .build(),
      NodeProperties::builder()
        .name("input_ports")
        .kind(NodePropertyKind::Number)
        .required(false)
        .display_name("输入端口数量")
        .description("期望的输入端口数量（2-10）")
        .value(json!(2))
        .build(),
    ])
    .build()
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::DataSource;
  use serde_json::json;

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = MergeNode::default();
    let metadata = node.definition();

    assert_eq!(metadata.kind.as_ref(), "Merge");
    assert_eq!(&metadata.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&metadata.display_name, "Merge");
    assert_eq!(&metadata.inputs.len(), &2);
    assert_eq!(&metadata.outputs.len(), &1);
  }

  #[test]
  fn test_merge_append() {
    let node = MergeNode::default();
    let current_node_id = NodeName::from("merge_append");

    let input_data = vec![
      ExecutionData {
        json: json!({"name": "Alice", "age": 25}),
        source: Some(DataSource {
          node_name: NodeName::from("true"),
          output_port: "true".to_string(),
          output_index: 0,
        }),
        index: 0,
        binary: None,
      },
      ExecutionData {
        json: json!({"name": "Bob", "age": 30}),
        source: Some(DataSource {
          node_name: NodeName::from("false"),
          output_port: "false".to_string(),
          output_index: 0,
        }),
        index: 1,
        binary: None,
      },
    ];

    let result = node.merge_append(&input_data, current_node_id).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].json["name"], "Alice");
    assert_eq!(result[1].json["name"], "Bob");
  }

  #[test]
  fn test_merge_by_key() {
    let node = MergeNode::default();
    let current_node_id = NodeName::from("merge_by_key");
    let merge_key = Some("id".to_string());

    let input_data = vec![
      ExecutionData { json: json!({"id": "1", "name": "Alice", "age": 25}), source: None, index: 0, binary: None },
      ExecutionData { json: json!({"id": "1", "city": "New York"}), source: None, index: 1, binary: None },
      ExecutionData { json: json!({"id": "2", "name": "Bob", "age": 30}), source: None, index: 2, binary: None },
    ];

    let result = node.merge_by_key(&input_data, &merge_key, current_node_id).unwrap();

    assert_eq!(result.len(), 2);

    // 查找 id=1 的合并结果
    let alice_record = result.iter().find(|r| r.json["id"] == "1").unwrap();
    assert_eq!(alice_record.json["name"], "Alice");
    assert_eq!(alice_record.json["city"], "New York");
  }

  #[test]
  fn test_merge_by_index() {
    let node = MergeNode::default();
    let current_node_id = NodeName::from("merge_by_index");

    let input_data = vec![
      ExecutionData { json: json!({"name": "Alice"}), source: None, index: 0, binary: None },
      ExecutionData {
        json: json!({"age": 25}),
        source: None,
        index: 0, // 相同索引
        binary: None,
      },
      ExecutionData { json: json!({"name": "Bob"}), source: None, index: 1, binary: None },
    ];

    let result = node.merge_by_index(&input_data, current_node_id).unwrap();

    assert_eq!(result.len(), 2);

    // 第一个结果应该合并了相同索引的数据
    assert_eq!(result[0].json["name"], "Alice");
    assert_eq!(result[0].json["age"], 25);
    assert_eq!(result[1].json["name"], "Bob");
  }

  #[test]
  fn test_merge_json_objects() {
    let node = MergeNode::default();

    let obj1 = json!({"name": "Alice", "age": 25});
    let obj2 = json!({"age": 26, "city": "New York"});

    let result = node.merge_json_objects(vec![&obj1, &obj2]).unwrap();

    assert_eq!(result["name"], "Alice");
    assert_eq!(result["age"], 26); // obj2 的值覆盖了 obj1
    assert_eq!(result["city"], "New York");
  }

  #[test]
  fn test_node_ports() {
    let node = MergeNode::default();

    let input_ports = &node.definition().inputs[..];
    assert_eq!(input_ports.len(), 2);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);
    assert_eq!(input_ports[1].kind, ConnectionKind::Main);

    let output_ports = &node.definition().outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }
}
