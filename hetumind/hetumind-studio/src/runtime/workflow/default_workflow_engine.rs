use std::sync::Arc;

use async_trait::async_trait;
use fusion_common::ahash::HashMap;
use fusion_common::time::now;
use fusion_core::application::Application;

use hetumind_core::{
  expression::ExpressionEvaluator,
  workflow::{
    ConnectionKind, EngineAction, EngineRequest, EngineResponse, EngineResult, ExecuteNodeAction, ExecutionContext,
    ExecutionData, ExecutionDataItems, ExecutionDataMap, ExecutionGraph, ExecutionId, ExecutionMetrics,
    ExecutionPlanner, ExecutionResult, ExecutionStatus, ExecutionTrace, GetConnectionDataAction, NodeExecutionContext,
    NodeExecutionResult, NodeExecutionStatus, NodeName, NodeRegistry, NodesExecutionMap, TriggerType, WorkflowEngine,
    WorkflowEngineSetting, WorkflowExecutionError, WorkflowTriggerData,
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
  /// 执行计划器
  execution_planner: ExecutionPlanner,
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
    let execution_planner = ExecutionPlanner::new();

    Self { node_registry, execution_store, scheduler, _concurrency_controller, _monitor, _config, execution_planner }
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
    println!("[DEBUG] 查找节点执行器: {} ({})", node_name, node.kind);
    let executor = self.node_registry.get_executor(&node.kind).ok_or(WorkflowExecutionError::NodeExecutionFailed {
      workflow_id: workflow.id.clone(),
      node_name: node_name.clone(),
    })?;
    println!("[DEBUG] 找到节点执行器: {} ({})", node_name, node.kind);

    // 2. 汇集父节点的输出
    let parents_results = collect_parents_results(node_name, graph, all_results);

    // 3. 创建节点执行上下文
    let node_context = make_node_context(context, node_name, parents_results, engine_response.cloned());

    // 4. 执行节点
    println!("[DEBUG] 开始执行节点: {} ({})", node_name, node.kind);
    println!("[DEBUG] 节点参数: {:?}", node.parameters);
    let output_data = executor.execute(&node_context).await.map_err(|e| {
      println!("[DEBUG] 节点 {} 执行失败: {:?}", node_name, e);
      println!("[DEBUG] 错误类型: {}", std::any::type_name_of_val(&e));
      println!("[DEBUG] 详细错误信息: {}", e);
      WorkflowExecutionError::NodeExecutionFailed { workflow_id: workflow.id.clone(), node_name: node_name.clone() }
    })?;
    println!("[DEBUG] 节点 {} 执行成功", node_name);

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
    let execution_data_vec = vec![ExecutionData::new_json(node_action.input.clone(), None)];
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
      .binary_data_manager(Application::global().component())
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
    .binary_data_manager(Application::global().component())
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
    trigger_data: WorkflowTriggerData,
    context: &ExecutionContext,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    // 统一的工作流执行路径
    let graph = ExecutionGraph::new(&context.workflow());

    if graph.has_cycles() {
      return Err(WorkflowExecutionError::CircularDependency);
    }

    // 根据触发类型准备执行数据
    let (node_name, execution_data) = match trigger_data.trigger_type {
      TriggerType::Normal { node_name, execution_data } => {
        // 正常工作流执行
        (node_name, execution_data)
      }
      TriggerType::Error { error_data, error_workflow_id: _ } => {
        // 错误工作流执行：将 WorkflowErrorData 转换为 ExecutionData
        let error_execution_data =
          ExecutionData::try_from(error_data.as_ref()).map_err(|e| WorkflowExecutionError::InternalError {
            message: format!("Failed to convert WorkflowErrorData to ExecutionData: {}", e),
          })?;
        let mut execution_data_map = HashMap::default();
        execution_data_map.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(vec![error_execution_data])]);

        // 使用默认的起始节点名称，或者可以从错误数据中推断
        let start_node = NodeName::from("start");
        (start_node, execution_data_map)
      }
    };

    // 使用统一的执行路径
    self.execute_with_engine_requests((node_name, execution_data), context, &graph).await
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

  // 实现新增的优化方法
  async fn get_execution_metrics(
    &self,
    execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionMetrics>, WorkflowExecutionError> {
    // 从存储中获取执行记录
    if let Some(execution) = self.execution_store.get_execution(execution_id).await? {
      // 计算执行时长
      let duration_ms = match (execution.started_at, execution.finished_at) {
        (Some(start), Some(end)) => {
          let duration = end.signed_duration_since(start);
          duration.num_milliseconds() as u64
        }
        _ => 0,
      };

      // 获取内存使用情况
      let memory_usage_mb = self.get_memory_usage().await?;
      let cpu_usage_percent = self.get_cpu_usage().await?;
      let cache_hit_rate = self.get_cache_hit_rate().await?;

      let metrics = ExecutionMetrics {
        execution_id: execution_id.clone(),
        duration_ms,
        nodes_executed: 0, // TODO: 从执行记录中获取详细信息
        nodes_succeeded: 0,
        nodes_failed: 0,
        memory_usage_mb,
        cpu_usage_percent,
        cache_hit_rate,
        retry_count: 0, // TODO: 从重试配置中获取
      };

      Ok(Some(metrics))
    } else {
      Ok(None)
    }
  }

  async fn get_execution_trace(
    &self,
    execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionTrace>, WorkflowExecutionError> {
    if let Some(execution) = self.execution_store.get_execution(execution_id).await? {
      let start_time = execution
        .started_at
        .map(|dt| chrono::DateTime::from_timestamp(dt.timestamp(), 0).unwrap_or_default().fixed_offset())
        .unwrap_or_else(|| chrono::Utc::now().fixed_offset());

      let end_time = execution
        .finished_at
        .map(|dt| Some(chrono::DateTime::from_timestamp(dt.timestamp(), 0).unwrap_or_default().fixed_offset()))
        .unwrap_or(Some(chrono::Utc::now().fixed_offset()));

      let trace = ExecutionTrace {
        execution_id: execution_id.clone(),
        start_time,
        end_time,
        node_traces: vec![],  // TODO: 从执行记录中构建节点追踪
        error_traces: vec![], // TODO: 收集错误追踪
      };

      Ok(Some(trace))
    } else {
      Ok(None)
    }
  }
}

impl DefaultWorkflowEngine {
  /// 获取内存使用情况
  async fn get_memory_usage(&self) -> Result<f64, WorkflowExecutionError> {
    // 简单实现，返回当前进程内存使用情况
    let memory_bytes = self.get_process_memory().unwrap_or(0);
    Ok(memory_bytes as f64 / (1024.0 * 1024.0)) // 转换为 MB
  }

  /// 获取 CPU 使用情况
  async fn get_cpu_usage(&self) -> Result<f64, WorkflowExecutionError> {
    // 简单实现，返回模拟 CPU 使用率
    Ok(50.0) // TODO: 实现真实的 CPU 监控
  }

  /// 获取缓存命中率
  async fn get_cache_hit_rate(&self) -> Result<f64, WorkflowExecutionError> {
    // 简单实现，返回模拟缓存命中率
    Ok(85.0) // TODO: 实现真实的缓存命中率统计
  }

  /// 获取进程内存使用情况
  fn get_process_memory(&self) -> Option<usize> {
    #[cfg(unix)]
    {
      use std::fs;
      let status = fs::read_to_string("/proc/self/status").ok()?;
      for line in status.lines() {
        if line.starts_with("VmRSS:") {
          let parts: Vec<&str> = line.split_whitespace().collect();
          if parts.len() >= 2 {
            return parts[1].parse::<usize>().ok();
          }
        }
      }
    }
    None
  }

  /// 估算数据大小
  fn estimate_data_size(&self, _data: &ExecutionDataMap) -> u64 {
    // 简单实现，返回估算的大小
    1024 // TODO: 实现真实的数据大小计算
  }

  /// 执行支持引擎请求的工作流
  async fn execute_with_engine_requests(
    &self,
    trigger_data: (NodeName, ExecutionDataMap),
    context: &ExecutionContext,
    graph: &ExecutionGraph,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    // 使用执行计划器进行并行执行
    if self._config.enable_parallel_execution {
      self.execute_workflow_parallel(trigger_data, context, graph).await
    } else {
      self.execute_workflow_sequential(trigger_data, context, graph).await
    }
  }

  /// 顺序执行工作流（原有实现）
  async fn execute_workflow_sequential(
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
          Err(e) => {
            log::error!("节点 {} 执行返回错误: {}", node_name, e);
            NodeExecutionResult::builder()
              .node_name(node_name.clone())
              .output_data(ExecutionDataMap::default())
              .status(NodeExecutionStatus::Failed)
              .error(e.to_string())
              .duration_ms(duration_ms)
              .build()
          }
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

    // 计算最终状态：如果任何节点失败，工作流状态为失败
    let final_status = if nodes_result.values().any(|r| r.status == NodeExecutionStatus::Failed) {
      ExecutionStatus::Failed
    } else {
      ExecutionStatus::Success
    };

    Ok(
      ExecutionResult::builder()
        .execution_id(context.execution_id().clone())
        .status(final_status)
        .nodes_result(nodes_result)
        .end_nodes(graph.get_end_nodes())
        .duration_ms(duration_ms)
        .build(),
    )
  }

  /// 并行执行工作流（新实现）
  async fn execute_workflow_parallel(
    &self,
    trigger_data: (NodeName, ExecutionDataMap),
    context: &ExecutionContext,
    graph: &ExecutionGraph,
  ) -> Result<ExecutionResult, WorkflowExecutionError> {
    // 生成执行计划
    let mut execution_plan = self.execution_planner.plan_execution(graph)?;
    self.execution_planner.optimize_execution_plan(&mut execution_plan)?;

    let mut all_results: NodesExecutionMap = HashMap::default();
    all_results.insert(trigger_data.0, trigger_data.1);

    let mut nodes_result: HashMap<NodeName, NodeExecutionResult> = HashMap::default();
    let mut engine_responses: HashMap<NodeName, EngineResponse> = HashMap::default();

    // 按照并行组执行
    for parallel_group in execution_plan.parallel_groups {
      if parallel_group.is_empty() {
        continue;
      }

      // 检查组内所有节点是否可以执行
      let mut ready_nodes = Vec::new();
      for node_name in &parallel_group {
        if self.can_execute_node(node_name, graph, &all_results) {
          ready_nodes.push(node_name.clone());
        }
      }

      if ready_nodes.is_empty() {
        continue;
      }

      // 执行组内节点（简化版：顺序执行，但组间按照依赖关系优化）
      for node_name in ready_nodes {
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
          Err(e) => {
            log::error!("节点 {} 执行返回错误: {}", node_name, e);
            NodeExecutionResult::builder()
              .node_name(node_name.clone())
              .output_data(ExecutionDataMap::default())
              .status(NodeExecutionStatus::Failed)
              .error(e.to_string())
              .duration_ms(duration_ms)
              .build()
          }
        };

        all_results.insert(node_name.clone(), node_execution_result.output_data.clone());
        nodes_result.insert(node_name.clone(), node_execution_result);
      }
    }

    let duration_ms = now().signed_duration_since(context.started_at()).num_milliseconds() as u64;

    // 计算最终状态：如果任何节点失败，工作流状态为失败
    let final_status = if nodes_result.values().any(|r| r.status == NodeExecutionStatus::Failed) {
      ExecutionStatus::Failed
    } else {
      ExecutionStatus::Success
    };

    Ok(
      ExecutionResult::builder()
        .execution_id(context.execution_id().clone())
        .status(final_status)
        .nodes_result(nodes_result)
        .end_nodes(graph.get_end_nodes())
        .duration_ms(duration_ms)
        .build(),
    )
  }

  /// 检查节点是否可以执行
  fn can_execute_node(&self, node_name: &NodeName, graph: &ExecutionGraph, all_results: &NodesExecutionMap) -> bool {
    if let Some(parent_names) = graph.get_parents(node_name) {
      parent_names.iter().all(|parent_name| all_results.contains_key(parent_name))
    } else {
      true // 无父节点，可以执行
    }
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
