use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition, NodeDefinitionBuilder,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{LoopConfig, LoopMode, utils::process_loop};

/// Loop Over Items 数据循环节点 V1
///
/// 用于对数据集合进行迭代处理，支持多种循环策略。
/// 常用于需要对数组或对象进行批量处理的场景。
///
/// # 循环模式
/// - `Items`: 对每个数据项执行一次循环
/// - `Times`: 固定次数循环
/// - `While`: 条件循环，直到条件不满足
/// - `Batch`: 批量处理，每次处理一批数据
///
/// # 输入端口
/// - 单个主输入端口，接收需要循环处理的数据
///
/// # 输出端口
/// - 单个主输出端口，包含所有循环执行的结果
#[derive(Debug)]
pub struct LoopV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl NodeExecutable for LoopV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Loop Over Items 节点 workflow_id:{}, node_name:{}, node_kind:{}",
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
      log::warn!("Loop Over Items 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取循环配置
    let mode: LoopMode = node.get_parameter("mode")?;
    let iterations: Option<u32> = node.get_optional_parameter("iterations");
    let batch_size: Option<usize> = node.get_optional_parameter("batch_size");
    let condition: Option<String> = node.get_optional_parameter("condition");
    let max_iterations: Option<u32> = node.get_optional_parameter("max_iterations");
    let include_index: bool = node.get_optional_parameter("include_index").unwrap_or(false);
    let parallel: bool = node.get_optional_parameter("parallel").unwrap_or(false);

    let config = LoopConfig { mode, iterations, batch_size, condition, max_iterations, include_index, parallel };

    // 验证配置
    if let Err(e) = config.validate() {
      return Err(NodeExecutionError::DataProcessingError { message: format!("Invalid loop configuration: {}", e) });
    }

    log::debug!(
      "循环配置: 模式={:?}, 输入数据量={}, 批量大小={:?}, 最大迭代次数={:?}",
      config.mode,
      input_items.len(),
      config.batch_size,
      config.max_iterations
    );

    // 执行循环处理
    let processed_data = process_loop(&input_items, &config, &context.current_node_name)?;

    log::info!("Loop Over Items 节点执行完成: 输入 {} 项，输出 {} 项", input_items.len(), processed_data.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(processed_data)])]))
  }
}

impl TryFrom<NodeDefinitionBuilder> for LoopV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .version(Version::new(1, 0, 0))
      .inputs([InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
      .outputs([OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
      .properties([
        NodeProperty::builder()
          .display_name("循环模式")
          .name("mode")
          .kind(NodePropertyKind::Options)
          .required(true)
          .description("循环执行策略")
          .value(json!(LoopMode::Items))
          .options(vec![
            Box::new(NodeProperty::new_option("Items", "items", json!(LoopMode::Items), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Times", "times", json!(LoopMode::Times), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("While", "while", json!(LoopMode::While), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Batch", "batch", json!(LoopMode::Batch), NodePropertyKind::String)),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("循环次数")
          .name("iterations")
          .kind(NodePropertyKind::Number)
          .required(false)
          .description("循环执行次数（仅 Times 模式）")
          .value(json!(1))
          .build(),
        NodeProperty::builder()
          .display_name("批量大小")
          .name("batch_size")
          .kind(NodePropertyKind::Number)
          .required(false)
          .description("每批处理的数据项数量（仅 Batch 模式）")
          .value(json!(10))
          .build(),
        NodeProperty::builder()
          .display_name("条件表达式")
          .name("condition")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("循环条件（仅 While 模式）")
          .placeholder("data.enabled")
          .build(),
        NodeProperty::builder()
          .display_name("最大循环次数")
          .name("max_iterations")
          .kind(NodePropertyKind::Number)
          .required(false)
          .description("防止无限循环的最大迭代次数")
          .value(json!(1000))
          .build(),
        NodeProperty::builder()
          .display_name("包含索引")
          .name("include_index")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("是否在输出中包含索引信息")
          .value(json!(false))
          .build(),
        NodeProperty::builder()
          .display_name("并行处理")
          .name("parallel")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("是否并行处理数据项")
          .value(json!(false))
          .build(),
      ]);

    let definition = base.build()?;

    Ok(Self { definition: Arc::new(definition) })
  }
}
