use std::sync::Arc;

use async_trait::async_trait;
use fusion_common::ahash::HashMap;
use fusion_common::time::now;
use fusion_core::application::Application;
use hetumind_core::binary_storage::BinaryDataManager;

use crate::runtime::workflow::EngineRouter;
use hetumind_core::{
  expression::ExpressionEvaluator,
  workflow::{
    ConnectionKind, ExecutionContext, ExecutionData, ExecutionDataItems, ExecutionDataMap, ExecutionGraph, ExecutionId,
    ExecutionMetrics, ExecutionPlanner, ExecutionResult, ExecutionStatus, ExecutionTrace, NodeExecutionContext,
    NodeExecutionResult, NodeExecutionStatus, NodeName, NodeRegistry, NodesExecutionMap, TriggerType, WorkflowEngine,
    WorkflowEngineSetting, WorkflowExecutionError, WorkflowTriggerData,
  },
};
use hetumind_nodes::common::helpers::get_simple_memory_supplier_typed;
use hetumind_nodes::store::simple_memory_node::SimpleMemorySupplier;
use serde_json::json;

use crate::runtime::{
  execution::ExecutionStore,
  monitor::ExecutionMonitor,
  task::{ConcurrencyController, TaskScheduler},
};

pub struct DefaultWorkflowEngine {
  /// 节点注册表
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
  ) -> Result<ExecutionDataMap, WorkflowExecutionError> {
    let workflow = context.workflow();
    let node = workflow.get_node(node_name).ok_or_else(|| WorkflowExecutionError::NodeExecutionFailed {
      workflow_id: workflow.id.clone(),
      node_name: node_name.clone(),
    })?;

    // 1. 查找节点执行器
    log::debug!("查找节点执行器: {} ({})", node_name, node.kind);
    let executor = self.node_registry.get_executor(&node.kind).ok_or(WorkflowExecutionError::NodeExecutionFailed {
      workflow_id: workflow.id.clone(),
      node_name: node_name.clone(),
    })?;
    log::debug!("找到节点执行器: {} ({})", node_name, node.kind);

    // 2. 汇集父节点的输出
    let mut parents_results = collect_parents_results(node_name, graph, all_results);

    // 在执行 LLM 节点前，尝试读取会话历史并注入到输入（system_prompt/messages），保持 NodeExecutionContext 不改动
    if self.node_registry.get_llm_supplier_typed(&node.kind).is_some() {
      // 查找输入数据（优先 AiLM，其次 Main）
      let maybe_items = parents_results
        .get(&ConnectionKind::AiLM)
        .cloned()
        .or_else(|| parents_results.get(&ConnectionKind::Main).cloned());

      if let Some(items_vec) = maybe_items {
        if let Some(first_items) = items_vec.first() {
          if let Some(data_vec) = first_items.get_data_items() {
            if let Some(input_data) = data_vec.first() {
              let mut input_json = input_data.json().clone();
              // 提取 session_id 与历史条数
              if let Some(session_id) = input_json.get("session_id").and_then(|v| v.as_str()) {
                let history_count = input_json.get("history_count").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                if let Some(mem_supplier) = get_simple_memory_supplier_typed(&self.node_registry) {
                  // 优先使用具体类型的 with_ctx 方法，失败则回退到 trait 方法
                  let mem_msgs = if let Some(simple) = mem_supplier.as_any().downcast_ref::<SimpleMemorySupplier>() {
                    simple.retrieve_messages_with_ctx(context, session_id, history_count).await.unwrap_or_default()
                  } else {
                    mem_supplier.retrieve_messages(session_id, history_count).await.unwrap_or_default()
                  };

                  // 构造历史上下文文本，注入到 system_prompt；同时合并到 messages（如存在）
                  if !mem_msgs.is_empty() {
                    let history_text = mem_msgs
                      .iter()
                      .map(|m| format!("- {}: {}", m.role, m.content))
                      .collect::<Vec<String>>()
                      .join("\n");

                    let new_system_prompt = match input_json.get("system_prompt").and_then(|v| v.as_str()) {
                      Some(sp) if !sp.is_empty() => format!("{}\n\n[History]\n{}", sp, history_text),
                      _ => format!("[History]\n{}", history_text),
                    };
                    input_json["system_prompt"] = json!(new_system_prompt);

                    // 合并到 messages：保留已有 messages，并追加历史
                    let mut messages =
                      input_json.get("messages").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    for m in mem_msgs.iter() {
                      messages.push(json!({"role": m.role, "content": m.content}));
                    }
                    input_json["messages"] = json!(messages);

                    // 追加 prompt 为用户消息（若存在）
                    if let Some(prompt) = input_json.get("prompt").and_then(|v| v.as_str()) {
                      let mut messages =
                        input_json.get("messages").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                      messages.push(json!({"role": "user", "content": prompt}));
                      input_json["messages"] = json!(messages);
                    }

                    // 注入历史条数字段（用于观测与复现）
                    input_json["history_length"] = json!(mem_msgs.len());

                    // 写回修改后的输入数据
                    let new_exec_data = ExecutionData::new_json(input_json, input_data.source().cloned());
                    // 更新 parents_results 中对应的端口数据
                    if parents_results.get(&ConnectionKind::AiLM).is_some() {
                      parents_results
                        .insert(ConnectionKind::AiLM, vec![ExecutionDataItems::new_items(vec![new_exec_data])]);
                    } else {
                      parents_results
                        .insert(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![new_exec_data])]);
                    }
                  }
                }
              }
            }
          }
        }
      }
    }

    // 3. 创建节点执行上下文
    let node_context = make_node_context(context, node_name, parents_results, self.node_registry.clone())?;

    // 4. 执行节点
    log::debug!("开始执行节点: {} ({})", node_name, node.kind);
    log::debug!("节点参数: {:?}", node.parameters);
    let mut output_data = executor.execute(&node_context).await.map_err(|e| {
      log::debug!("节点 {} 执行失败: {:?}", node_name, e);
      log::debug!("错误类型: {}", std::any::type_name_of_val(&e));
      log::debug!("详细错误信息: {}", e);
      WorkflowExecutionError::NodeExecutionFailed { workflow_id: workflow.id.clone(), node_name: node_name.clone() }
    })?;
    log::debug!("节点 {} 执行成功", node_name);

    // 5. 处理 EngineRequest（AiTool 端口），通过 EngineRouter 统一路由到对应 Tool 节点执行
    let router = EngineRouter::new(self.node_registry.clone());
    router.route_engine_requests(&mut output_data, context).await?;

    Ok(output_data)
  }
}

fn make_node_context(
  context: &ExecutionContext,
  node_name: &NodeName,
  parents_results: ExecutionDataMap,
  node_registry: NodeRegistry,
) -> Result<NodeExecutionContext, WorkflowExecutionError> {
  // 获取 BinaryDataManager 组件
  let binary_data_manager = Application::global()
    .get_component::<BinaryDataManager>()
    .map_err(|e| WorkflowExecutionError::InternalError { message: format!("BinaryDataManager not found: {}", e) })?;

  let ctx = NodeExecutionContext::new(
    *context.execution_id(),
    context.workflow(),
    node_name.clone(),
    parents_results,
    binary_data_manager,
    node_registry,
  )
  .with_started_at(now())
  .with_user_id(context.ctx().user_id())
  .with_env_vars(std::env::vars())
  .with_expression_evaluator(ExpressionEvaluator::new());

  Ok(ctx)
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
    let graph = ExecutionGraph::new(&context.workflow(), &self.node_registry);

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
    self.execute_workflow_parallel((node_name, execution_data), context, &graph).await
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
        execution_id: *execution_id,
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
        execution_id: *execution_id,
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

  /// 并行执行工作流（真正的并行执行）
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

    // 按照并行组执行 - 真正的并行执行
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

      // 并行执行组内节点
      let mut node_futures = Vec::new();
      for node_name in ready_nodes {
        let node_future = {
          let node_name = node_name.clone();
          let graph = &graph;
          let all_results = &all_results;
          async move { self.execute_node_in_parallel(node_name, graph, all_results, context).await }
        };
        node_futures.push(node_future);
      }

      // 等待所有节点完成
      let node_results = futures::future::join_all(node_futures).await;

      // 处理并行执行结果
      for node_result in node_results {
        match node_result {
          Ok(result) => {
            // 使用节点的实际输出数据
            all_results.insert(result.node_name.clone(), result.output_data.clone());
            nodes_result.insert(result.node_name.clone(), result);
          }
          Err(e) => {
            log::error!("并行执行节点失败: {}", e);
            // 并行执行中错误信息已经在 execute_node_in_parallel 中处理
          }
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

    Ok(ExecutionResult::new(*context.execution_id(), final_status, graph.get_end_nodes(), duration_ms, nodes_result))
  }

  /// 并行执行单个节点
  async fn execute_node_in_parallel(
    &self,
    node_name: NodeName,
    graph: &ExecutionGraph,
    all_results: &NodesExecutionMap,
    context: &ExecutionContext,
  ) -> Result<NodeExecutionResult, WorkflowExecutionError> {
    let started_at = now();

    let result = self.execute_single_node(&node_name, graph, all_results, context).await;
    let duration_ms = now().signed_duration_since(started_at).num_milliseconds() as u64;

    let node_execution_result = match result {
      Ok(output_data) => NodeExecutionResult::success(node_name, duration_ms, output_data),
      Err(e) => NodeExecutionResult::failure(node_name, duration_ms, NodeExecutionStatus::Failed, e.to_string()),
    };

    Ok(node_execution_result)
  }

  /// 检查节点是否可以执行
  fn can_execute_node(&self, node_name: &NodeName, graph: &ExecutionGraph, all_results: &NodesExecutionMap) -> bool {
    if let Some(parent_names) = graph.get_parents(node_name) {
      parent_names.iter().all(|parent_name| all_results.contains_key(parent_name))
    } else {
      true // 无父节点，可以执行
    }
  }
}
