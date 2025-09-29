use std::{sync::Arc, time::Duration};

use hetumind_core::workflow::{
  ExecutionContext, ExecutionDataCollection, ExecutionId, NodeExecutable, NodeExecutionContext, NodeExecutionError,
  NodeRegistry, WorkflowNode,
};

use crate::runtime::monitor::ExecutionMonitor;

pub struct NodeExecutableImpl {
  /// 节点类型注册表
  node_kinds: NodeRegistry,
  /// 执行器线程池
  executor_pool: Arc<tokio::runtime::Handle>,
  /// 监控器
  monitor: Arc<ExecutionMonitor>,
}

impl NodeExecutableImpl {
  pub async fn execute_node(
    &self,
    node: &WorkflowNode,
    context: &ExecutionContext,
  ) -> Result<Vec<ExecutionDataCollection>, NodeExecutionError> {
    let start_time = std::time::Instant::now();

    // 获取节点执行器
    let executor = self
      .node_kinds
      .get_executor(&node.kind)
      .ok_or_else(|| NodeExecutionError::UnsupportedNodeKind { node_kind: node.kind.clone() })?;

    // TODO 检验参数

    // 创建执行上下文
    let node_context = self.create_node_context(node, context).await?;

    // 执行节点
    let result = self.execute_with_timeout(executor, &node_context).await;

    let duration = start_time.elapsed();
    // 记录执行指标
    self.monitor.record_node_execution(&node.name, duration, &result).await;

    result
  }

  async fn create_node_context(
    &self,
    node: &WorkflowNode,
    context: &ExecutionContext,
  ) -> Result<NodeExecutionContext, NodeExecutionError> {
    // TODO 创建节点执行上下文
    Ok(NodeExecutionContext {
      execution_id: ExecutionId::now_v7(),
      workflow: todo!(),
      current_node_name: node.name,
      input_data: todo!(),
      started_at: todo!(),
      user_id: todo!(),
      env_vars: todo!(),
    })
  }

  async fn execute_with_timeout(
    &self,
    executor: NodeExecutor,
    context: &NodeExecutionContext,
  ) -> Result<Vec<ExecutionDataCollection>, NodeExecutionError> {
    let node = context.current_node()?;
    let timeout_duration = Duration::from_secs(node.timeout.unwrap_or(300)); // TODO 默认5分钟

    tokio::time::timeout(timeout_duration, executor.execute(context))
      .await
      .map_err(|_| NodeExecutionError::Timeout)?
  }
}
