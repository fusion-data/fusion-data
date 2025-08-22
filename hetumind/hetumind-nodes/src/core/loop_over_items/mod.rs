//! LoopOverItems 循环批处理节点实现
//!
//! 参考 n8n 的 Split In Batches (Loop Over Items) 节点设计，用于将大量数据分割成较小的批次进行逐批处理。
//! 这是工作流中重要的循环控制节点，能够优化大数据集的处理性能。

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::workflow::{
  ConnectionKind, DataSource, ExecutionData, InputPortConfig, NodeDefinition, NodeExecutionContext, NodeExecutionError,
  NodeExecutor, NodeGroupKind, NodeKind, NodeProperties, NodePropertyKind, OutputPortConfig, ValidationError,
  WorkflowNode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use log::{debug, info, warn};

use crate::constants::LOOP_OVER_ITEMS_NODE_KIND;

/// 批次元数据信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMetadata {
  /// 当前批次索引（从0开始）
  pub batch_index: usize,
  /// 总批次数
  pub total_batches: usize,
  /// 当前批次大小
  pub batch_size: usize,
  /// 当前批次中的元素数量
  pub current_batch_count: usize,
  /// 是否为第一批
  pub is_first_batch: bool,
  /// 是否为最后一批
  pub is_last_batch: bool,
  /// 已处理的总元素数量
  pub processed_count: usize,
  /// 剩余元素数量
  pub remaining_count: usize,
}

/// 批次配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
  /// 每个批次的大小
  pub batch_size: usize,
  /// 批次间暂停时间（毫秒）
  pub pause_between_batches: Option<u64>,
  /// 是否在失败时继续处理下一批
  pub continue_on_fail: Option<bool>,
  /// 是否包含批次元数据
  pub include_metadata: Option<bool>,
}

/// LoopOverItems 循环批处理节点
///
/// 用于将输入的数组数据分割成多个批次，支持逐批处理大量数据。
/// 主要用于优化大数据集的处理性能，避免内存溢出和超时问题。
///
/// # 主要功能
/// - 将数组数据分割成指定大小的批次
/// - 为每个批次提供元数据信息（索引、总数、是否最后一批等）
/// - 支持批次间暂停和错误处理策略
/// - 保持数据来源信息和索引关系
///
/// # 使用场景
/// - 处理大量 API 调用
/// - 批量数据库操作
/// - 大文件分块处理
/// - 避免 API 速率限制
///
/// # 输入/输出
/// - 输入：包含数组的 JSON 数据
/// - 输出：每个批次的数据和元数据信息
#[derive(Debug, Clone)]
pub struct LoopOverItemsNode {
  definition: Arc<NodeDefinition>,
}

impl Default for LoopOverItemsNode {
  fn default() -> Self {
    let definition = Arc::new(
      NodeDefinition::builder()
        .kind(NodeKind::from(LOOP_OVER_ITEMS_NODE_KIND))
        .versions(vec![1])
        .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output])
        .display_name("Loop Over Items")
        .description("将数据分割成批次进行循环处理。用于优化大数据集的处理性能。")
        .icon("repeat".to_string())
        .inputs(vec![InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input 1").build()])
        .outputs(vec![OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output 1").build()])
        .properties(vec![
          NodeProperties::builder()
            .name("batch_size".to_string())
            .kind(NodePropertyKind::Number)
            .required(true)
            .display_name("批次大小")
            .description("每个批次包含的元素数量")
            .value(json!(10))
            .placeholder("10")
            .build(),
          NodeProperties::builder()
            .name("pause_between_batches".to_string())
            .kind(NodePropertyKind::Number)
            .required(false)
            .display_name("批次间暂停时间（毫秒）")
            .description("每个批次处理之间的暂停时间，用于避免API速率限制")
            .value(json!(0))
            .placeholder("0")
            .build(),
          NodeProperties::builder()
            .name("continue_on_fail".to_string())
            .kind(NodePropertyKind::Boolean)
            .required(false)
            .display_name("失败时继续")
            .description("当某个批次处理失败时是否继续处理下一批次")
            .value(json!(true))
            .placeholder("true")
            .build(),
          NodeProperties::builder()
            .name("include_metadata".to_string())
            .kind(NodePropertyKind::Boolean)
            .required(false)
            .display_name("包含元数据")
            .description("是否在输出中包含批次元数据信息")
            .value(json!(true))
            .placeholder("true")
            .build(),
        ])
        .build(),
    );
    Self { definition }
  }
}

impl LoopOverItemsNode {
  /// 从节点参数中获取批次参数
  fn get_batch_config(&self, node: &WorkflowNode) -> Result<BatchConfig, ValidationError> {
    let config: BatchConfig = node.parameters.get()?;
    Ok(config)
  }

  /// 将数据分割成批次
  fn split_into_batches(&self, data: &Value, config: &BatchConfig) -> Result<Vec<Value>, NodeExecutionError> {
    // 提取数组数据
    let items = self.extract_array_items(data)?;

    if items.is_empty() {
      warn!("输入数据为空数组");
      return Ok(vec![]);
    }

    let total_items = items.len();
    let total_batches = total_items.div_ceil(config.batch_size); // 向上取整
    let mut batches = Vec::new();

    debug!("开始分批: 总元素={}, 批次大小={}, 总批次={}", total_items, config.batch_size, total_batches);

    // 分割数据
    for batch_index in 0..total_batches {
      let start_index = batch_index * config.batch_size;
      let end_index = std::cmp::min(start_index + config.batch_size, total_items);

      let batch_items = &items[start_index..end_index];
      let current_batch_count = batch_items.len();

      // 创建批次元数据
      let metadata = BatchMetadata {
        batch_index,
        total_batches,
        batch_size: config.batch_size,
        current_batch_count,
        is_first_batch: batch_index == 0,
        is_last_batch: batch_index == total_batches - 1,
        processed_count: end_index,
        remaining_count: total_items - end_index,
      };

      // 创建批次数据
      let mut batch_data = json!({
        "items": batch_items,
      });

      // 如果配置包含元数据，则添加元数据信息
      if config.include_metadata.unwrap_or(true) {
        batch_data["metadata"] = json!(metadata);
      }

      batches.push(batch_data);

      debug!("创建批次 {}/{}: {} 个元素", batch_index + 1, total_batches, current_batch_count);
    }

    Ok(batches)
  }

  /// 提取数组项目
  fn extract_array_items(&self, data: &Value) -> Result<Vec<Value>, NodeExecutionError> {
    match data {
      // 如果输入直接是数组
      Value::Array(arr) => Ok(arr.clone()),

      // 如果输入是对象，尝试查找数组字段
      Value::Object(obj) => {
        // 常见的数组字段名
        let possible_array_fields = ["items", "data", "results", "list", "values"];

        for field_name in &possible_array_fields {
          if let Some(field_value) = obj.get(*field_name) {
            if let Some(arr) = field_value.as_array() {
              return Ok(arr.clone());
            }
          }
        }

        // 如果没有找到明显的数组字段，将对象本身作为单个元素
        Ok(vec![data.clone()])
      }

      // 其他类型作为单个元素处理
      _ => Ok(vec![data.clone()]),
    }
  }

  /// 获取数组总数
  #[allow(dead_code)]
  fn get_total_count(&self, data: &Value) -> usize {
    match data {
      Value::Array(arr) => arr.len(),
      Value::Object(obj) => {
        let possible_array_fields = ["items", "data", "results", "list", "values"];
        for field_name in &possible_array_fields {
          if let Some(field_value) = obj.get(*field_name) {
            if let Some(arr) = field_value.as_array() {
              return arr.len();
            }
          }
        }
        1 // 对象本身算作1个元素
      }
      _ => 1, // 其他类型算作1个元素
    }
  }
}

#[async_trait]
impl NodeExecutor for LoopOverItemsNode {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<Vec<Vec<ExecutionData>>, NodeExecutionError> {
    let node = context.current_node()?;
    info!(
      "开始执行 LoopOverItems 循环批处理节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id, node.name, node.kind
    );

    // 获取输入数据
    if context.input_data.is_empty() {
      warn!("LoopOverItems 节点没有接收到输入数据");
      return Ok(vec![]);
    }

    // 获取批次配置
    let config = self.get_batch_config(node)?;

    debug!("批次配置: 批次大小={}, 暂停={}ms", config.batch_size, config.pause_between_batches.unwrap_or(0));

    let mut all_results = Vec::new();

    // 处理每个输入数据项
    for input_collection in context.get_input_items(ConnectionKind::Main, 0) {
      let input_items = input_collection
        .get_data_items()
        .ok_or_else(|| NodeExecutionError::InvalidInputData { connection_kind: ConnectionKind::Main, port_index: 0 })?;

      for input_data in input_items.iter() {
        let batches = self.split_into_batches(&input_data.json, &config)?;

        debug!("数据分批完成: {} 个批次", batches.len());

        // 处理每个批次
        for (batch_index, batch_data) in batches.into_iter().enumerate() {
          // 添加批次间暂停
          if batch_index > 0 {
            if let Some(pause_ms) = config.pause_between_batches {
              if pause_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(pause_ms)).await;
              }
            }
          }

          all_results.push(ExecutionData {
            json: batch_data,
            source: Some(DataSource {
              node_name: context.current_node_name.clone(),
              output_port: "main".to_string(),
              output_index: batch_index,
            }),
            index: input_data.index,
            binary: input_data.binary.clone(),
          });
        }
      }
    }

    info!("LoopOverItems 节点执行完成: 生成 {} 个批次", all_results.len());

    Ok(vec![all_results])
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = LoopOverItemsNode::default();
    let metadata = node.definition();

    assert_eq!(metadata.kind.as_ref(), "LoopOverItems");
    assert_eq!(&metadata.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&metadata.display_name, "Loop Over Items");
    assert_eq!(metadata.inputs.len(), 1);
    assert_eq!(metadata.outputs.len(), 1);
    assert_eq!(metadata.properties.len(), 4);
  }

  #[test]
  fn test_extract_array_items() {
    let node = LoopOverItemsNode::default();

    // 测试直接数组
    let array_data = json!([1, 2, 3, 4, 5]);
    let items = node.extract_array_items(&array_data).unwrap();
    assert_eq!(items.len(), 5);

    // 测试包含 items 字段的对象
    let object_data = json!({
      "items": [1, 2, 3],
      "other": "data"
    });
    let items = node.extract_array_items(&object_data).unwrap();
    assert_eq!(items.len(), 3);

    // 测试没有数组字段的对象
    let single_object = json!({"name": "test", "value": 42});
    let items = node.extract_array_items(&single_object).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], single_object);

    // 测试基本类型
    let primitive_data = json!("hello");
    let items = node.extract_array_items(&primitive_data).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], "hello");
  }

  #[test]
  fn test_split_into_batches() {
    let node = LoopOverItemsNode::default();
    let config =
      BatchConfig { batch_size: 3, pause_between_batches: None, continue_on_fail: None, include_metadata: Some(true) };

    // 测试正好整除的情况
    let data = json!([1, 2, 3, 4, 5, 6]);
    let batches = node.split_into_batches(&data, &config).unwrap();
    assert_eq!(batches.len(), 2);

    // 检查第一批
    assert_eq!(batches[0]["items"], json!([1, 2, 3]));
    assert_eq!(batches[0]["metadata"]["batch_index"], json!(0));
    assert_eq!(batches[0]["metadata"]["total_batches"], json!(2));
    assert_eq!(batches[0]["metadata"]["is_first_batch"], json!(true));
    assert_eq!(batches[0]["metadata"]["is_last_batch"], json!(false));

    // 检查第二批
    assert_eq!(batches[1]["items"], json!([4, 5, 6]));
    assert_eq!(batches[1]["metadata"]["batch_index"], json!(1));
    assert_eq!(batches[1]["metadata"]["is_first_batch"], json!(false));
    assert_eq!(batches[1]["metadata"]["is_last_batch"], json!(true));
  }

  #[test]
  fn test_split_into_batches_remainder() {
    let node = LoopOverItemsNode::default();
    let config =
      BatchConfig { batch_size: 3, pause_between_batches: None, continue_on_fail: None, include_metadata: Some(true) };

    // 测试有余数的情况
    let data = json!([1, 2, 3, 4, 5]);
    let batches = node.split_into_batches(&data, &config).unwrap();
    assert_eq!(batches.len(), 2);

    // 检查第一批
    assert_eq!(batches[0]["items"], json!([1, 2, 3]));
    assert_eq!(batches[0]["metadata"]["current_batch_count"], json!(3));

    // 检查第二批（不满的批次）
    assert_eq!(batches[1]["items"], json!([4, 5]));
    assert_eq!(batches[1]["metadata"]["current_batch_count"], json!(2));
    assert_eq!(batches[1]["metadata"]["is_last_batch"], json!(true));
  }

  #[test]
  fn test_split_into_batches_without_metadata() {
    let node = LoopOverItemsNode::default();
    let config =
      BatchConfig { batch_size: 2, pause_between_batches: None, continue_on_fail: None, include_metadata: Some(false) };

    let data = json!([1, 2, 3, 4]);
    let batches = node.split_into_batches(&data, &config).unwrap();
    assert_eq!(batches.len(), 2);

    // 检查不包含元数据
    assert!(batches[0].get("metadata").is_none());
    assert_eq!(batches[0]["items"], json!([1, 2]));
    assert_eq!(batches[1]["items"], json!([3, 4]));
  }

  #[test]
  fn test_empty_array() {
    let node = LoopOverItemsNode::default();
    let config =
      BatchConfig { batch_size: 3, pause_between_batches: None, continue_on_fail: None, include_metadata: Some(true) };

    let data = json!([]);
    let batches = node.split_into_batches(&data, &config).unwrap();
    assert_eq!(batches.len(), 0);
  }

  #[test]
  fn test_node_ports() {
    let node = LoopOverItemsNode::default();

    let input_ports = &node.definition().inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &node.definition().outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_get_total_count() {
    let node = LoopOverItemsNode::default();

    // 测试数组
    let array_data = json!([1, 2, 3, 4, 5]);
    assert_eq!(node.get_total_count(&array_data), 5);

    // 测试包含数组的对象
    let object_data = json!({
      "items": [1, 2, 3],
      "other": "data"
    });
    assert_eq!(node.get_total_count(&object_data), 3);

    // 测试单个对象
    let single_object = json!({"name": "test"});
    assert_eq!(node.get_total_count(&single_object), 1);

    // 测试基本类型
    let primitive = json!("hello");
    assert_eq!(node.get_total_count(&primitive), 1);
  }
}
