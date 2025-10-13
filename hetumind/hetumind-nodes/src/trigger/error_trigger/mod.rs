//! # ErrorTriggerNode
//!
//! A node that triggers workflow execution when other workflows fail.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeExecutable, NodeExecutionContext,
  NodeExecutionError, NodeExecutor, NodeKind, RegistrationError, make_execution_data_map,
};

mod parameters;
mod utils;

pub struct ErrorTriggerNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for ErrorTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for ErrorTriggerNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // Error 触发器作为入口点，通常由错误处理框架调用
    // 在手动模式下生成示例数据用于开发和测试

    let is_manual_test = utils::is_manual_test_mode(context);

    if is_manual_test {
      // 生成示例错误数据用于测试
      let error_data = utils::generate_sample_error_data();
      let execution_data = hetumind_core::workflow::ExecutionData::new_json(error_data, None);
      let items = vec![ExecutionDataItems::new_items(vec![execution_data])];
      return Ok(make_execution_data_map(vec![(ConnectionKind::Main, items)]));
    }

    // 在实际错误触发场景中，错误数据由触发器框架提供
    // 这里返回空数据，实际错误数据通过上下文传入
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

pub struct ErrorTriggerNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl Node for ErrorTriggerNode {
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

impl ErrorTriggerNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = utils::create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(ErrorTriggerNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}
