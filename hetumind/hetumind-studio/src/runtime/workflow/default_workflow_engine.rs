use std::sync::Arc;

use ahash::HashMap;
use async_trait::async_trait;
use fusion_common::time::now;
use hetumind_core::{
  expression::ExpressionEvaluator,
  workflow::{
    ConnectionKind, EngineAction, EngineRequest, EngineResponse, EngineResult, ExecuteNodeAction, ExecutionContext,
    ExecutionData, ExecutionDataItems, ExecutionDataMap, ExecutionGraph, ExecutionId, ExecutionResult, ExecutionStatus,
    GetConnectionDataAction, NodeExecutionContext, NodeExecutionResult, NodeExecutionStatus, NodeName, NodeRegistry,
    NodesExecutionMap, WorkflowEngine, WorkflowEngineSetting, WorkflowErrorData, WorkflowExecutionError, WorkflowId,
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
    engine_response: Option<&EngineResponse>,
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
    let node_context = make_node_context(context, node_name, parents_results, engine_response.cloned());

    // 4. 执行节点
    let output_data = executor.execute(&node_context).await.map_err(|_| {
      WorkflowExecutionError::NodeExecutionFailed { workflow_id: workflow.id.clone(), node_name: node_name.clone() }
    })?;

    Ok(output_data)
  }

  /// 处理引擎请求
  async fn handle_engine_request(
    &self,
    request: EngineRequest,
    context: &ExecutionContext,
  ) -> Result<EngineResponse, WorkflowExecutionError> {
    let mut action_responses = Vec::new();

    for action in request.actions {
      match action {
        EngineAction::ExecuteNode(node_action) => {
          let result = self.execute_node_action(node_action, context).await?;
          action_responses.push(result);
        }
        EngineAction::GetConnectionData(data_action) => {
          let result = self.get_connection_data_action(data_action, context).await?;
          action_responses.push(result);
        }
      }
    }

    Ok(EngineResponse { action_responses, metadata: Default::default(), response_id: request.request_id })
  }

  /// 执行节点动作
  async fn execute_node_action(
    &self,
    node_action: ExecuteNodeAction,
    context: &ExecutionContext,
  ) -> Result<EngineResult, WorkflowExecutionError> {
    let workflow = context.workflow();
    let node_name = NodeName::from(node_action.node_name.clone());

    // 创建临时执行数据
    let mut input_data = ExecutionDataMap::default();
    let mut execution_data_vec = Vec::new();
    execution_data_vec.push(ExecutionData::new_json(node_action.input.clone(), None));
    input_data.insert(node_action.connection_type, vec![ExecutionDataItems::Items(execution_data_vec)]);

    // 查找节点执行器
    let node = workflow.get_node(&node_name).ok_or_else(|| WorkflowExecutionError::NodeExecutionFailed {
      workflow_id: workflow.id.clone(),
      node_name: node_name.clone(),
    })?;

    let executor =
      self
        .node_registry
        .get_executor(&node.kind)
        .ok_or_else(|| WorkflowExecutionError::NodeExecutionFailed {
          workflow_id: workflow.id.clone(),
          node_name: node_name.clone(),
        })?;

    // 创建节点执行上下文
    let node_context = NodeExecutionContext::builder()
      .execution_id(context.execution_id().clone())
      .workflow(context.workflow())
      .current_node_name(node_name.clone())
      .input_data(input_data)
      .started_at(now())
      .user_id(Some(context.ctx().uid()))
      .env_vars(std::env::vars().collect())
      .expression_evaluator(ExpressionEvaluator::new())
      .build();

    // 执行节点
    let output_data = executor.execute(&node_context).await.map_err(|_| {
      WorkflowExecutionError::NodeExecutionFailed { workflow_id: workflow.id.clone(), node_name: node_name.clone() }
    })?;

    Ok(EngineResult {
      action: EngineAction::ExecuteNode(node_action),
      data: output_data,
      status: NodeExecutionStatus::Success,
      error: None,
    })
  }

  /// 获取连接数据动作
  async fn get_connection_data_action(
    &self,
    _data_action: GetConnectionDataAction,
    _context: &ExecutionContext,
  ) -> Result<EngineResult, WorkflowExecutionError> {
    // TODO: 实现获取连接数据的逻辑
    Ok(EngineResult {
      action: EngineAction::GetConnectionData(_data_action),
      data: ExecutionDataMap::default(),
      status: NodeExecutionStatus::Success,
      error: None,
    })
  }
}

fn make_node_context(
  context: &ExecutionContext,
  node_name: &NodeName,
  parents_results: ExecutionDataMap,
  engine_response: Option<EngineResponse>,
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
    .engine_response(engine_response)
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
  async fn execute_error_workflow(
    &self,
    error_data: WorkflowErrorData,
    error_workflow_id: Option<WorkflowId>,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    todo!()
  }

  async fn execute_workflow(
    &self,
    trigger_data: (NodeName, ExecutionDataMap),
    context: &ExecutionContext,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    let graph = ExecutionGraph::new(&context.workflow());

    if graph.has_cycles() {
      return Err(WorkflowExecutionError::CircularDependency);
    }

    // 执行支持引擎请求的工作流
    self.execute_with_engine_requests(trigger_data, context, &graph).await
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

impl DefaultWorkflowEngine {
  /// 执行支持引擎请求的工作流
  async fn execute_with_engine_requests(
    &self,
    trigger_data: (NodeName, ExecutionDataMap),
    context: &ExecutionContext,
    graph: &ExecutionGraph,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    let mut all_results: NodesExecutionMap = HashMap::default();
    all_results.insert(trigger_data.0, trigger_data.1);

    let mut nodes_result: HashMap<NodeName, NodeExecutionResult> = HashMap::default();
    let mut pending_nodes = graph.get_start_nodes();
    let mut engine_responses: HashMap<NodeName, EngineResponse> = HashMap::default();

    while !pending_nodes.is_empty() {
      let nodes = std::mem::take(&mut pending_nodes);

      for node_name in nodes {
        let started_at = now();

        // 检查是否有待处理的引擎请求
        let engine_response = engine_responses.remove(&node_name);

        let execute_result =
          self.execute_single_node(&node_name, graph, &all_results, context, engine_response.as_ref()).await;
        let duration_ms = now().signed_duration_since(started_at).num_milliseconds() as u64;

        let node_execution_result = match execute_result {
          Ok(output_data) => {
            // 检查是否返回了引擎请求
            if let Some(engine_request) = self.extract_engine_request(&output_data) {
              // 处理引擎请求
              match self.handle_engine_request(engine_request, context).await {
                Ok(response) => {
                  // 将响应存储以便后续节点使用
                  engine_responses.insert(node_name.clone(), response);

                  NodeExecutionResult::builder()
                    .node_name(node_name.clone())
                    .output_data(output_data)
                    .status(NodeExecutionStatus::Success)
                    .duration_ms(duration_ms)
                    .build()
                }
                Err(e) => NodeExecutionResult::builder()
                  .node_name(node_name.clone())
                  .output_data(ExecutionDataMap::default())
                  .status(NodeExecutionStatus::Failed)
                  .error(e.to_string())
                  .duration_ms(duration_ms)
                  .build(),
              }
            } else {
              NodeExecutionResult::builder()
                .node_name(node_name.clone())
                .output_data(output_data)
                .status(NodeExecutionStatus::Success)
                .duration_ms(duration_ms)
                .build()
            }
          }
          Err(e) => NodeExecutionResult::builder()
            .node_name(node_name.clone())
            .output_data(ExecutionDataMap::default())
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

  /// 从输出数据中提取引擎请求
  fn extract_engine_request(&self, output_data: &ExecutionDataMap) -> Option<EngineRequest> {
    // 检查 tool_calls 端口是否有引擎请求
    if let Some(tool_calls_data) = output_data.get(&ConnectionKind::AiTool)
      && let Some(first_item) = tool_calls_data.first()
      && let Some(data_items) = first_item.get_data_items()
      && let Some(first_data) = data_items.first()
      && let Ok(engine_request) = serde_json::from_value::<EngineRequest>(first_data.json().clone())
    {
      return Some(engine_request);
    }
    None
  }
}
