use std::sync::Arc;

use async_trait::async_trait;

use super::{ExecutionDataMap, NodeDefinition, NodeExecutionContext, NodeExecutionError};

#[async_trait]
pub trait NodeExecutor {
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
