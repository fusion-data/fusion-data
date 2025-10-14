//! # ManualTriggerNode
//!
//! 手动触发器节点，允许用户通过手动方式触发工作流执行。
//! 相比 StartNode，ManualTriggerNode 提供了更丰富的配置选项，
//! 包括执行模式、启用状态等。

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, NodeProperty, NodePropertyKind,
  RegistrationError, make_execution_data_map,
};
use serde_json::json;

use crate::constants::MANUAL_TRIGGER_NODE_KIND;
use parameters::ManualTriggerConfig;

mod parameters;

pub struct ManualTriggerNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for ManualTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(base: NodeDefinition) -> Result<Self, Self::Error> {
    let definition = base;
    Ok(Self { definition: Arc::new(definition) })
  }
}

pub fn create_base() -> NodeDefinition {
  NodeDefinition::new(MANUAL_TRIGGER_NODE_KIND, Version::new(1, 0, 0), "Manual Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("手动触发工作流执行，支持执行模式和启用状态配置")
    .add_property(
      NodeProperty::builder()
        .display_name("操作提示")
        .name("notice")
        .kind(NodePropertyKind::Notice)
        .description("这是工作流执行的起点，点击'执行工作流'按钮来触发工作流")
        .value(json!("点击执行工作流按钮来启动工作流"))
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("执行模式")
        .name("execution_mode")
        .kind(NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option("测试模式", "test", json!("test"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option("生产模式", "production", json!("production"), NodePropertyKind::String)),
        ])
        .required(true)
        .description("选择工作流执行模式")
        .value(json!("test"))
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("启用状态")
        .name("enabled")
        .kind(NodePropertyKind::Boolean)
        .required(false)
        .description("是否启用手动触发功能")
        .value(json!(true))
        .build(),
    )
}

#[async_trait]
impl NodeExecutable for ManualTriggerNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.definition)
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // 1. 获取当前节点信息
    let node = context.current_node()?;

    // 2. 解析配置参数
    let config = self.parse_config(&node.parameters)?;

    // 3. 检查是否启用
    if !config.enabled {
      return Err(NodeExecutionError::ParameterValidation(
        hetumind_core::workflow::ValidationError::invalid_field_value("enabled", "手动触发器已禁用"),
      ));
    }

    // 4. 生成触发数据
    let trigger_data = config.generate_trigger_data();
    let trigger_id = trigger_data["trigger_id"].as_str().unwrap_or("unknown").to_string();

    // 5. 创建执行数据项
    let data_items = ExecutionDataItems::new_items(vec![ExecutionData::new_json(trigger_data, None)]);

    // 6. 记录执行日志
    log::info!(
      "Manual trigger executed - node: {}, execution_mode: {:?}, trigger_id: {}",
      node.name,
      config.execution_mode,
      trigger_id
    );

    // 7. 返回执行数据映射
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![data_items])]))
  }
}

impl ManualTriggerNodeV1 {
  /// 解析节点参数
  fn parse_config(
    &self,
    parameters: &hetumind_core::workflow::ParameterMap,
  ) -> Result<ManualTriggerConfig, NodeExecutionError> {
    // 直接反序列化整个参数映射到配置结构体
    let config: ManualTriggerConfig = parameters.get().map_err(|e| {
      NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::invalid_field_value(
        "parameters",
        format!("参数解析失败: {}", e),
      ))
    })?;

    Ok(config)
  }
}

pub struct ManualTriggerNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl Node for ManualTriggerNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

impl ManualTriggerNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(ManualTriggerNodeV1::try_from(base)?)];
    Ok(Self { default_version: Version::new(1, 0, 0), executors })
  }
}
