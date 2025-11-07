//! # ScheduleTriggerNode
//!
//! A node that triggers workflow execution on a scheduled basis.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ExecutionDataItems, ExecutionDataMap, FlowNode, FlowNodeRef, Node, NodeConnectionKind, NodeDescription,
  NodeExecutionContext, NodeExecutionError, NodeType, RegistrationError, make_execution_data_map,
};

mod parameters;
mod utils;

use parameters::*;

pub struct ScheduleTriggerNodeV1 {
  definition: Arc<NodeDescription>,
}

impl TryFrom<NodeDescription> for ScheduleTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDescription) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl FlowNode for ScheduleTriggerNodeV1 {
  fn description(&self) -> Arc<NodeDescription> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // Schedule 触发器作为入口点，返回空数据
    // 实际的调度逻辑在触发器框架层面完成

    let node = context.current_node()?;
    let _paramters = parse_schedule_parameters(&node.parameters)?;

    // TODO: 解析参数，根据模式执行调度
    // 1. 如果是 cron 表达式，解析并设置 cron 调度
    // 2. 如果是 interval 表达式，解析并设置固定时间间隔调度

    Ok(make_execution_data_map(vec![(NodeConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

pub struct ScheduleTriggerNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl Node for ScheduleTriggerNode {
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

impl ScheduleTriggerNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = utils::create_base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(ScheduleTriggerNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.description().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}
