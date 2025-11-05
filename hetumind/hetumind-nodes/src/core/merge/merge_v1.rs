use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition, FlowNode,
    NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig, RegistrationError,
    make_execution_data_map,
  },
};
use serde_json::json;

use super::{MergeConfig, MergeMode, utils::merge_data};

/// Merge 数据合并节点 V1
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
#[derive(Debug)]
pub struct MergeV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl FlowNode for MergeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Merge 数据合并节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      input_data
    } else {
      log::warn!("Merge 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取合并配置
    let mode: MergeMode = node.get_parameter("mode")?;
    let merge_key: Option<String> = node.get_optional_parameter("merge_key");
    let input_ports: Option<usize> = node.get_optional_parameter("input_ports");

    let config = MergeConfig { mode, merge_key, input_ports };

    // 验证配置
    if let Err(e) = config.validate() {
      return Err(NodeExecutionError::DataProcessingError { message: format!("Invalid merge configuration: {}", e) });
    }

    log::debug!("合并配置: 模式={:?}, 合并键={:?}, 输入数据量={}", config.mode, config.merge_key, input_items.len());

    // 执行合并操作
    let merged_data = merge_data(&input_items, &config, &context.current_node_name)?;

    log::info!("Merge 节点执行完成: 输入 {} 项，输出 {} 项", input_items.len(), merged_data.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(merged_data)])]))
  }
}

impl TryFrom<NodeDefinition> for MergeV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(ConnectionKind::Main, "Input"))
      .add_output(OutputPortConfig::new(ConnectionKind::Main, "Output"))
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("合并模式")
          .with_name("mode")
          .with_required(true)
          .with_description("数据合并策略")
          .with_value(json!(MergeMode::Append))
          .with_options(vec![
            Box::new(NodeProperty::new_option("Append", "append", json!(MergeMode::Append), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "MergeByKey",
              "merge_by_key",
              json!(MergeMode::MergeByKey),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "MergeByIndex",
              "merge_by_index",
              json!(MergeMode::MergeByIndex),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "WaitForAll",
              "wait_for_all",
              json!(MergeMode::WaitForAll),
              NodePropertyKind::String,
            )),
          ]),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("合并键")
          .with_name("merge_key")
          .with_required(false)
          .with_description("用于按键合并的字段名（仅 mergeByKey 模式）")
          .with_placeholder("id"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("输入端口数量")
          .with_name("input_ports")
          .with_required(false)
          .with_description("期望的输入端口数量（2-10）")
          .with_value(json!(2)),
      );
    Ok(Self { definition: Arc::new(definition) })
  }
}
