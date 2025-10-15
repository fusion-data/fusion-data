use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{KeepStrategy, LimitConfig};

#[derive(Debug)]
pub struct LimitV1 {
  pub definition: Arc<NodeDefinition>,
}

impl LimitV1 {
  /// 应用限制操作到数据项
  pub fn apply_limit(&self, input_items: &[ExecutionData], config: &LimitConfig) -> Vec<ExecutionData> {
    // 检查是否需要应用限制
    if input_items.len() <= config.max_items {
      log::info!("输入项目数量 {} 不超过限制 {}，返回所有项目", input_items.len(), config.max_items);
      return input_items.to_vec();
    }

    if config.warn_on_limit {
      log::warn!(
        "输入项目数量 {} 超过限制 {}，将应用限制策略: {:?}",
        input_items.len(),
        config.max_items,
        config.keep_strategy
      );
    }

    // 直接在这里实现简单的切片操作
    match config.keep_strategy {
      KeepStrategy::FirstItems => input_items[..config.max_items].to_vec(),
      KeepStrategy::LastItems => {
        let start_index = input_items.len().saturating_sub(config.max_items);
        input_items[start_index..].to_vec()
      }
    }
  }
}

/// Limit 限制节点
///
/// 用于限制通过的数据项数量，支持保留前 N 个或后 N 个项目。
/// 基于 n8n Limit Node 的简洁设计理念，提供高效的数据流控制。
///
/// # 功能特性
/// - 简单高效的数组切片操作
/// - 支持保留前 N 个或后 N 个项目
/// - 早期退出优化，避免不必要的处理
/// - 详细的执行日志和统计信息
///
/// # 使用场景
/// - API 响应数量限制
/// - 批处理大小控制
/// - 测试数据采样
/// - 性能优化控制
#[async_trait]
impl NodeExecutable for LimitV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 Limit 节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Limit 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::error!("Limit 节点没有接收到输入数据");
      // 如果没有输入数据，返回空结果
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Vec::new())],
      )]));
    };

    // 获取限制配置
    let max_items: usize = node.get_parameter("max_items")?;
    let keep_strategy: KeepStrategy = node.get_optional_parameter("keep_strategy").unwrap_or(KeepStrategy::FirstItems);
    let warn_on_limit: bool = node.get_optional_parameter("warn_on_limit").unwrap_or(true);

    let config = LimitConfig { max_items, keep_strategy, warn_on_limit };

    // 验证配置
    if let Err(e) = config.validate() {
      log::error!("Limit 配置验证失败: {:?}", e);
      return Err(NodeExecutionError::ExecutionFailed {
        node_name: node.name.clone().into(),
        message: Some(format!("Limit 配置验证失败: {:?}", e)),
      });
    }

    log::info!(
      "Limit 配置 - 最大项目数: {}, 保留策略: {:?}, 警告开关: {}",
      config.max_items,
      config.keep_strategy,
      config.warn_on_limit
    );

    // 执行限制操作
    let result_items = self.apply_limit(&input_items, &config);

    // 记录执行统计
    log::info!(
      "Limit 执行完成 - 输入: {} 项, 输出: {} 项, 限制: {} 项, 策略: {:?}",
      input_items.len(),
      result_items.len(),
      config.max_items,
      config.keep_strategy
    );

    // 返回结果
    let res = vec![ExecutionDataItems::new_items(result_items)];
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, res)]))
  }
}

impl TryFrom<NodeDefinition> for LimitV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base
      .with_version(Version::new(1, 0, 0))
      .add_input(InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build())
      .add_output(OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build())
      .add_property(
        NodeProperty::builder()
          .display_name("Max Items".to_string())
          .name("max_items")
          .kind(NodePropertyKind::Number)
          .required(true)
          .description("If there are more items than this number, some are removed".to_string())
          .placeholder("1".to_string())
          .value(json!(1))
          .build(),
      )
      .add_property(
        NodeProperty::builder()
          .display_name("Keep".to_string())
          .name("keep_strategy")
          .kind(NodePropertyKind::Options)
          .required(false)
          .description("When removing items, whether to keep the ones at the start or the ending".to_string())
          .value(json!(KeepStrategy::FirstItems))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "First Items",
              "first_items",
              json!(KeepStrategy::FirstItems),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Last Items",
              "last_items",
              json!(KeepStrategy::LastItems),
              NodePropertyKind::String,
            )),
          ])
          .build(),
      )
      .add_property(
        NodeProperty::builder()
          .display_name("Warn on Limit".to_string())
          .name("warn_on_limit")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("Whether to log a warning when the limit is applied".to_string())
          .value(json!(true))
          .build(),
      );
    Ok(Self { definition: Arc::new(definition) })
  }
}
