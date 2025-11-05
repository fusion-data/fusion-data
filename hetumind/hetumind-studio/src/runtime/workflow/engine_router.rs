//! Engine Router：统一的 EngineRequest → Tool 路由执行器
//!
//! 用途：
//! - 在节点产生 AiTool 端口输出的 EngineRequest 时，解析请求并路由到对应 Tool 节点执行
//! - 将 Tool 执行结果封装为 EngineResponse，写回到 AiTool 端口
//!
//! 约束：
//! - 不引入审计或迁移逻辑
//! - 函数级注释，Rust 2024，2 空格缩进

use fusion_common::time::now;
use fusion_core::application::Application;
use hetumind_core::binary_storage::BinaryDataManager;
use hetumind_core::expression::ExpressionEvaluator;
use hetumind_core::workflow::{
  ConnectionKind, EngineRequest, EngineResponse, ExecutionContext, ExecutionData, ExecutionDataItems, ExecutionDataMap,
  Message, NodeExecutionContext, NodeName, NodeRegistry,
};
use hetumind_nodes::common::helpers::get_simple_memory_supplier_typed;
use hetumind_nodes::store::simple_memory_node::SimpleMemorySupplier;
use log::warn;
use std::collections::HashSet;
use tokio::time::{Duration, sleep};

use crate::runtime::workflow::WorkflowEnginePlugin; // 预留：后续路由可接入插件体系

/// 引擎路由器：持有必要的依赖，用于处理 EngineRequest
pub struct EngineRouter {
  node_registry: NodeRegistry,
}

impl EngineRouter {
  /// 创建新的 EngineRouter
  pub fn new(node_registry: NodeRegistry) -> Self {
    Self { node_registry }
  }

  /// 处理输出数据中的 EngineRequest 并执行对应 Tool 节点，将结果写回 AiTool 端口
  pub async fn route_engine_requests(
    &self,
    output_data: &mut ExecutionDataMap,
    context: &ExecutionContext,
  ) -> Result<(), hetumind_core::workflow::WorkflowExecutionError> {
    if let Some(items) = output_data.get(&ConnectionKind::AiTool) {
      let mut responses: Vec<ExecutionData> = Vec::new();
      let workflow = context.workflow();
      // 本次路由调用内的幂等处理：对 (correlation_id, id) 做去重
      let mut dedup: HashSet<String> = HashSet::new();

      for (idx, item) in items.iter().enumerate() {
        if let Some(data_items) = item.get_data_items() {
          for data in data_items {
            let json_val = data.json().clone();
            match serde_json::from_value::<EngineRequest>(json_val) {
              Ok(req) => {
                let dedup_key = format!("{}:{}", req.correlation_id.clone().unwrap_or_default(), req.id);
                if dedup.contains(&dedup_key) {
                  let event = serde_json::json!({
                    "event": "tool_call_dedup",
                    "status": "skipped",
                    "request_id": req.id,
                    "correlation_id": req.correlation_id,
                  });
                  log::info!("{}", event.to_string());
                  continue;
                }
                dedup.insert(dedup_key);
                // 查找目标 Tool 节点
                if let Some(tool_node) = workflow.get_node(&NodeName::from(req.node_name.as_str())) {
                  // 构造输入：将 EngineRequest.input 放到 Main 端口
                  let mut input_map: ExecutionDataMap = ExecutionDataMap::default();
                  let input_item = ExecutionData::new_json(req.input.clone(), None);
                  input_map.insert(ConnectionKind::Main, vec![ExecutionDataItems::new_item(input_item)]);

                  // 创建 Tool 节点上下文并执行
                  let binary_data_manager =
                    Application::global().get_component::<BinaryDataManager>().map_err(|e| {
                      hetumind_core::workflow::WorkflowExecutionError::InternalError {
                        message: format!("BinaryDataManager not found: {}", e),
                      }
                    })?;

                  let tool_context = NodeExecutionContext::new(
                    *context.execution_id(),
                    context.workflow(),
                    tool_node.name.clone(),
                    input_map,
                    binary_data_manager,
                    self.node_registry.clone(),
                  )
                  .with_started_at(now())
                  .with_user_id(context.ctx().user_id())
                  .with_env_vars(std::env::vars())
                  .with_expression_evaluator(ExpressionEvaluator::new());

                  if let Some(tool_executor) = self.node_registry.get_executor(&tool_node.kind) {
                    // 支持重试策略
                    let mut attempt = 0u32;
                    let max_retries = req.retry_policy.as_ref().map(|p| p.max_retries).unwrap_or(0);
                    let mut backoff_ms = req.retry_policy.as_ref().map(|p| p.initial_backoff_ms).unwrap_or(0);
                    let backoff_mul = req.retry_policy.as_ref().map(|p| p.backoff_multiplier).unwrap_or(1.0);

                    loop {
                      match tool_executor.execute(&tool_context).await {
                        Ok(tool_outputs) => {
                          // 取 Main 输出作为 EngineResponse.output（若无则返回空对象）
                          let mut output_json = serde_json::json!({});
                          if let Some(tool_items) = tool_outputs.get(&ConnectionKind::Main) {
                            if let Some(v) = tool_items.first().and_then(|x| x.get_data_items()) {
                              if let Some(first) = v.first() {
                                output_json = first.json().clone();
                              }
                            }
                          }
                          // 可选：将工具输出写入会话内存（角色：tool），需存在 session_id
                          if let Some(session_id) = req.metadata.get("session_id").and_then(|v| v.as_str()) {
                            if let Some(mem_supplier) = get_simple_memory_supplier_typed(&self.node_registry) {
                              let output_json_clone = output_json.clone();
                              let content = output_json_clone
                                .get("content")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| output_json_clone.to_string());
                              let msg = Message { role: "tool".to_string(), content };
                              // 优先使用具体类型的 with_ctx 方法，失败则回退到 trait 方法
                              if let Some(simple) = mem_supplier.as_any().downcast_ref::<SimpleMemorySupplier>() {
                                let _ = simple.store_messages_with_ctx(context, session_id, vec![msg]).await;
                              } else {
                                let _ = mem_supplier.store_messages(session_id, vec![msg]).await;
                              }
                            }
                          }

                          let resp = EngineResponse {
                            id: req.id.clone(),
                            output: output_json,
                            error: None,
                            correlation_id: req.correlation_id.clone(),
                          };
                          responses.push(ExecutionData::new_json(serde_json::to_value(resp).unwrap_or_default(), None));
                          // 结构化事件日志：工具执行成功
                          let event = serde_json::json!({
                            "event": "tool_call",
                            "status": "success",
                            "tool_node": tool_node.name.as_str(),
                            "request_id": req.id,
                            "correlation_id": req.correlation_id,
                          });
                          log::info!("{}", event.to_string());
                          break;
                        }
                        Err(err) => {
                          if attempt < max_retries {
                            attempt += 1;
                            if backoff_ms > 0 {
                              sleep(Duration::from_millis(backoff_ms)).await;
                            }
                            backoff_ms = (backoff_ms as f64 * backoff_mul) as u64;
                            continue;
                          }
                          let resp = EngineResponse {
                            id: req.id.clone(),
                            output: serde_json::json!({}),
                            error: Some(err.to_string()),
                            correlation_id: req.correlation_id.clone(),
                          };
                          responses.push(ExecutionData::new_json(serde_json::to_value(resp).unwrap_or_default(), None));
                          // 可选：将错误写入会话内存（角色：tool），需存在 session_id
                          if let Some(session_id) = req.metadata.get("session_id").and_then(|v| v.as_str()) {
                            if let Some(mem_supplier) = get_simple_memory_supplier_typed(&self.node_registry) {
                              let msg = Message { role: "tool".to_string(), content: format!("error: {}", err) };
                              if let Some(simple) = mem_supplier.as_any().downcast_ref::<SimpleMemorySupplier>() {
                                let _ = simple.store_messages_with_ctx(context, session_id, vec![msg]).await;
                              } else {
                                let _ = mem_supplier.store_messages(session_id, vec![msg]).await;
                              }
                            }
                          }
                          // 结构化事件日志：工具执行失败
                          let event = serde_json::json!({
                            "event": "tool_call",
                            "status": "error",
                            "tool_node": tool_node.name.as_str(),
                            "request_id": req.id,
                            "correlation_id": req.correlation_id,
                            "error": err.to_string(),
                            "error_code": Self::categorize_error_code(&err),
                            "attempt": attempt,
                          });
                          log::warn!("{}", event.to_string());
                          break;
                        }
                      }
                    }
                  } else {
                    let resp = EngineResponse {
                      id: req.id.clone(),
                      output: serde_json::json!({}),
                      error: Some(format!("Tool executor not found for node: {}", tool_node.name)),
                      correlation_id: req.correlation_id.clone(),
                    };
                    responses.push(ExecutionData::new_json(serde_json::to_value(resp).unwrap_or_default(), None));
                  }
                } else {
                  let resp = EngineResponse {
                    id: req.id.clone(),
                    output: serde_json::json!({}),
                    error: Some(format!("Tool node not found: {}", req.node_name)),
                    correlation_id: req.correlation_id.clone(),
                  };
                  responses.push(ExecutionData::new_json(serde_json::to_value(resp).unwrap_or_default(), None));
                }
              }
              Err(_) => {
                warn!("AiTool 输出项不是 EngineRequest 格式，索引 {} 被忽略", idx);
              }
            }
          }
        }
      }
      if !responses.is_empty() {
        output_data.insert(ConnectionKind::AiTool, vec![ExecutionDataItems::new_items(responses)]);
      }
    }

    Ok(())
  }

  /// 错误分类映射：统一错误语义用于观测与重试策略
  fn categorize_error_code(err: &hetumind_core::workflow::NodeExecutionError) -> &'static str {
    use hetumind_core::workflow::NodeExecutionError as E;
    match err {
      E::ParameterValidation(_) => "parameter_validation",
      E::InitFailed { .. } => "init_failed",
      E::InvalidInputData { .. } => "invalid_input_data",
      E::UnsupportedNodeKind { .. } => "unsupported_node",
      E::ExternalServiceError { .. } => "external_service",
      E::Timeout => "timeout",
      E::ExecutionFailed { .. } => "execution_failed",
      E::NodeNotFound { .. } => "node_not_found",
      E::DataProcessingError { .. } => "data_processing",
      E::ResourceExhausted => "resource_exhausted",
      E::InvalidInput(_) => "invalid_input",
      E::ConfigurationError(_) => "configuration_error",
      E::ConnectionError(_) => "connection_error",
    }
  }
}
