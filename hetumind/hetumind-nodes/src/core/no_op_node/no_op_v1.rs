use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ExecutionData, ExecutionDataItems, ExecutionDataMap, FlowNode, InputPortConfig, NodeConnectionKind,
    NodeDescription, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{NoOpConfig, utils};

#[derive(Debug)]
pub struct NoOpV1 {
  pub definition: Arc<NodeDescription>,
}

impl NoOpV1 {
  /// 执行 No Operation - 原样传递输入数据
  async fn execute_no_operation(
    &self,
    context: &NodeExecutionContext,
    input_items: &[ExecutionData],
    config: &NoOpConfig,
  ) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 NoOp 节点 workflow_id:{}, node_name:{}, node_type:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    log::info!("NoOp 节点接收到 {} 个输入项", input_items.len());

    // 如果启用了日志记录，记录数据传递信息
    if config.enable_logging {
      for (i, item) in input_items.iter().enumerate() {
        log::info!("传递输入项 {}: {}", i, utils::format_data_summary(item));
      }
    }

    // 如果启用了性能指标，记录执行时间
    let start_time = if config.enable_metrics { Some(std::time::Instant::now()) } else { None };

    // NoOp 的核心逻辑：原样传递输入数据
    let output_items = input_items.to_vec();

    // 记录性能指标
    if let (Some(start_time), true) = (start_time, config.enable_metrics) {
      let duration = start_time.elapsed();
      log::info!("NoOp 节点执行耗时: {:?}", duration);
      log::info!("处理数据项数量: {}", output_items.len());
    }

    log::info!("NoOp 节点执行完成 - 输入项: {}, 输出项: {}", input_items.len(), output_items.len());

    // 返回原样的数据
    Ok(make_execution_data_map(vec![(NodeConnectionKind::Main, vec![ExecutionDataItems::new_items(output_items)])]))
  }
}

/// NoOp 数据传递节点 V1
///
/// 实现最简单的节点功能：原样传递输入数据，不进行任何转换。
///
/// # 技术特点
/// - 零数据转换开销
/// - 最小内存占用
/// - O(n) 时间复杂度，其中 n 为输入数据项数量
/// - 支持可选的日志记录和性能监控
///
/// # 使用场景
/// - 工作流调试：作为数据检查点
/// - 工作流组织：分隔不同的工作流段
/// - 条件分支：作为占位符节点
/// - 数据验证：检查数据流转是否正确
#[async_trait]
impl FlowNode for NoOpV1 {
  fn description(&self) -> Arc<NodeDescription> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 NoOp 节点 workflow_id:{}, node_name:{}, node_type:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(NodeConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("NoOp 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::warn!("NoOp 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(
        NodeConnectionKind::Main,
        vec![ExecutionDataItems::new_items(vec![])],
      )]));
    };

    // 获取配置参数
    let enable_logging: bool = node.get_optional_parameter("enable_logging").unwrap_or(false);
    let enable_metrics: bool = node.get_optional_parameter("enable_metrics").unwrap_or(false);

    let config = NoOpConfig { enable_logging, enable_metrics };

    log::info!("NoOp 配置: 日志记录={}, 性能监控={}", enable_logging, enable_metrics);

    // 执行 No Operation
    self.execute_no_operation(context, &input_items, &config).await
  }
}

impl TryFrom<NodeDescription> for NoOpV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDescription) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::new(NodeConnectionKind::Main, "Input"))
      .add_output(OutputPortConfig::new(NodeConnectionKind::Main, "Output"))
      .add_property(
        // 调试选项
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Enable Logging")
          .with_name("enable_logging")
          .with_required(false)
          .with_description("Enable detailed logging of data passing through the node")
          .with_value(json!(false)),
      )
      .add_property(
        // 性能监控选项
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Enable Metrics")
          .with_name("enable_metrics")
          .with_required(false)
          .with_description("Enable performance metrics collection")
          .with_value(json!(false)),
      );
    Ok(Self { definition: Arc::new(definition) })
  }
}
