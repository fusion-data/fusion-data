use std::sync::Arc;

use ahash::HashMap;
use async_trait::async_trait;
use fusion_common::time::now;
use hetumind_core::{
  expression::ExpressionEvaluator,
  workflow::{
    ExecutionContext, ExecutionDataMap, ExecutionGraph, ExecutionId, ExecutionResult, ExecutionStatus,
    NodeExecutionContext, NodeExecutionResult, NodeExecutionStatus, NodeName, NodeRegistry, NodesExecutionMap,
    WorkflowEngine, WorkflowEngineSetting, WorkflowExecutionError,
  },
};

use crate::runtime::{
  execution::ExecutionStore,
  monitor::ExecutionMonitor,
  task::{ConcurrencyController, TaskScheduler},
};

pub struct DefaultWorkflowEngine {
  /// 节点执行器注册表
  node_registry: NodeRegistry,
  /// 执行状态存储
  execution_store: Arc<dyn ExecutionStore>,
  /// 任务调度器
  scheduler: Arc<TaskScheduler>,
  /// 并发控制器
  _concurrency_controller: Arc<ConcurrencyController>,
  /// 监控器
  _monitor: Arc<ExecutionMonitor>,
  /// 配置
  _config: WorkflowEngineSetting,
}

impl DefaultWorkflowEngine {
  pub fn new(
    node_registry: NodeRegistry,
    execution_store: Arc<dyn ExecutionStore>,
    _config: WorkflowEngineSetting,
  ) -> Self {
    let scheduler = Arc::new(TaskScheduler::new(_config.clone()));
    let _concurrency_controller = Arc::new(ConcurrencyController::new(_config.clone()));
    let _monitor = Arc::new(ExecutionMonitor::new());

    Self { node_registry, execution_store, scheduler, _concurrency_controller, _monitor, _config }
  }

  /// 执行单个节点
  async fn execute_single_node(
    &self,
    node_name: &NodeName,
    graph: &ExecutionGraph,
    all_results: &NodesExecutionMap,
    context: &ExecutionContext,
  ) -> Result<ExecutionDataMap, WorkflowExecutionError> {
    let workflow = context.workflow();
    let node = workflow.get_node(node_name).ok_or_else(|| WorkflowExecutionError::NodeExecutionFailed {
      workflow_id: workflow.id.clone(),
      node_name: node_name.clone(),
    })?;

    // 1. 查找节点执行器
    let executor = self.node_registry.get_executor(&node.kind).ok_or(WorkflowExecutionError::NodeExecutionFailed {
      workflow_id: workflow.id.clone(),
      node_name: node_name.clone(),
    })?;

    // 2. 汇集父节点的输出
    let parents_results = collect_parents_results(node_name, graph, all_results);

    // 3. 创建节点执行上下文
    let node_context = make_node_context(context, node_name, parents_results);

    // 4. 执行节点
    let output_data = executor.execute(&node_context).await.map_err(|_| {
      WorkflowExecutionError::NodeExecutionFailed { workflow_id: workflow.id.clone(), node_name: node_name.clone() }
    })?;

    Ok(output_data)
  }
}

fn make_node_context(
  context: &ExecutionContext,
  node_name: &NodeName,
  parents_results: ExecutionDataMap,
) -> NodeExecutionContext {
  NodeExecutionContext::builder()
    .execution_id(context.execution_id().clone())
    .workflow(context.workflow())
    .current_node_name(node_name.clone())
    .input_data(parents_results)
    .started_at(now())
    .user_id(Some(context.ctx().uid()))
    .env_vars(std::env::vars().collect())
    .expression_evaluator(ExpressionEvaluator::new())
    .build()
}

fn collect_parents_results(
  node_name: &NodeName,
  graph: &ExecutionGraph,
  all_results: &NodesExecutionMap,
) -> ExecutionDataMap {
  let mut parents_results: ExecutionDataMap = ExecutionDataMap::default();
  if let Some(parent_names) = graph.get_parents(node_name) {
    for parent_name in parent_names {
      if let Some(parent_outputs) = all_results.get(parent_name) {
        for (conn_kind, output_data) in parent_outputs {
          let outputs = parents_results.entry(*conn_kind).or_default();
          outputs.extend(output_data.clone());
        }
      }
    }
  }
  parents_results
}

// TODO: 若存在有多个开始节点的情况，需要考虑如何处理？
#[async_trait]
impl WorkflowEngine for DefaultWorkflowEngine {
  async fn execute_workflow(
    &self,
    trigger_data: (NodeName, ExecutionDataMap),
    context: &ExecutionContext,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    let graph = ExecutionGraph::new(&context.workflow());

    if graph.has_cycles() {
      return Err(WorkflowExecutionError::CircularDependency);
    }

    let mut all_results: NodesExecutionMap = HashMap::default();
    all_results.insert(trigger_data.0, trigger_data.1);

    let mut nodes_result: HashMap<NodeName, NodeExecutionResult> = HashMap::default();

    let mut pending_nodes = graph.get_start_nodes();

    while !pending_nodes.is_empty() {
      let nodes = std::mem::take(&mut pending_nodes);
      // TODO: 这里需要考虑节点执行顺序、错误处理，以及并发执行
      for node_name in nodes {
        let started_at = now();
        let execute_result = self.execute_single_node(&node_name, &graph, &all_results, context).await;
        let duration_ms = now().signed_duration_since(started_at).num_milliseconds() as u64;
        let node_execution_result = match execute_result {
          Ok(output_data) => NodeExecutionResult::builder()
            .node_name(node_name.clone())
            .output_data(output_data)
            .status(NodeExecutionStatus::Success)
            .duration_ms(duration_ms)
            .build(),
          Err(e) => NodeExecutionResult::builder()
          .node_name(node_name.clone())
          .output_data(ExecutionDataMap::default()) // TODO: 根据错误策略生成输出数据
          .status(NodeExecutionStatus::Failed)
          .error(e.to_string())
          .duration_ms(duration_ms)
          .build(),
        };
        all_results.insert(node_name.clone(), node_execution_result.output_data.clone());
        nodes_result.insert(node_name.clone(), node_execution_result);

        if let Some(children) = graph.get_children(&node_name)
          && !children.is_empty()
        {
          // TODO: 判断当前节点与子节点是否有正确的连接，如果有才加入到 pending_nodes 中
          pending_nodes.extend(children.clone());
        }
      }
    }

    let duration_ms = now().signed_duration_since(context.started_at()).num_milliseconds() as u64;
    Ok(
      ExecutionResult::builder()
        .execution_id(context.execution_id().clone())
        .status(ExecutionStatus::Success)
        .nodes_result(nodes_result)
        .end_nodes(graph.get_end_nodes())
        .duration_ms(duration_ms)
        .build(),
    )
  }

  async fn pause_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError> {
    self.scheduler.pause_execution(execution_id).await
  }

  async fn resume_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError> {
    self.scheduler.resume_execution(execution_id).await
  }

  async fn cancel_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError> {
    self.scheduler.cancel_execution(execution_id).await
  }

  async fn get_execution_status(&self, execution_id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError> {
    self.execution_store.get_execution_status(execution_id).await
  }
}
