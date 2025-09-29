use std::sync::Arc;

use async_trait::async_trait;

use crate::version::Version;
use crate::workflow::{ExecutionDataMap, NodeDefinition, NodeExecutionContext, NodeExecutionError, NodeKind};

#[async_trait]
pub trait NodeExecutable {
  /// 初始化节点。可用于实现节点初始化逻辑，如加载配置、初始化资源等。
  async fn init(&mut self, _context: &NodeExecutionContext) -> Result<(), NodeExecutionError> {
    Ok(())
  }

  /// 执行节点
  ///
  /// Returns:
  /// - 成功返回多个输出端口的数据，第 1 个输出端口从 0 开始
  /// - 失败返回错误
  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError>;

  /// 获取节点定义
  fn definition(&self) -> Arc<NodeDefinition>;
}

pub type NodeExecutor = Arc<dyn NodeExecutable + Send + Sync>;

pub trait Node {
  fn default_version(&self) -> &Version;

  fn node_executors(&self) -> &[NodeExecutor];

  fn kind(&self) -> NodeKind;

  fn versions(&self) -> Vec<Version> {
    self.node_executors().iter().map(|node| node.definition().version.clone()).collect()
  }

  fn get_node_executor(&self, version: &Version) -> Option<NodeExecutor> {
    self.node_executors().iter().find(|node| node.definition().version == *version).cloned()
  }

  fn default_node_executor(&self) -> Option<NodeExecutor> {
    self.get_node_executor(self.default_version())
  }
}
