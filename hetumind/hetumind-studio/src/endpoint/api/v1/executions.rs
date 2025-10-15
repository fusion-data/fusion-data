use axum::{
  Router,
  extract::Path,
  response::{Json, Sse, sse::Event},
  routing::{get, post},
};
use futures::stream::{Stream, StreamExt};
use fusion_core::application::Application;
use fusion_web::{WebResult, ok_json};
use fusionsql::page::PageResult;
use hetumind_core::workflow::{Execution, ExecutionData, ExecutionForQuery, ExecutionId, ExecutionStatus};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;

use crate::domain::workflow::ExecutionSvc;

pub fn routes() -> Router<Application> {
  Router::new()
    .route("/query", post(query_executions))
    .route("/{id}", get(get_execution))
    .route("/{id}/cancel", post(cancel_execution))
    .route("/{id}/retry", post(retry_execution))
    .route("/{id}/logs", get(get_execution_logs))
    .route("/{id}/status", get(get_execution_status))
    .route("/{id}/logs/stream", get(stream_execution_logs))
}

// --- API Data Models ---

/// The standard response for a single execution.
/// It uses the core execution model directly.
pub type ExecutionResponse = Execution;

/// The response for execution logs.
pub type ExecutionLogResponse = Vec<ExecutionData>;

/// Lightweight execution status response
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStatusResponse {
  /// 执行ID
  pub id: ExecutionId,
  /// 执行状态
  pub status: ExecutionStatus,
  /// 开始时间
  pub started_at: Option<fusion_common::time::OffsetDateTime>,
  /// 结束时间
  pub finished_at: Option<fusion_common::time::OffsetDateTime>,
  /// 错误信息（如果有）
  pub error: Option<String>,
  /// 执行进度（可选）
  pub progress: Option<f32>,
}

// --- API Handlers ---

/// 查询执行历史
pub async fn query_executions(
  execution_svc: ExecutionSvc,
  Json(input): Json<ExecutionForQuery>,
) -> WebResult<PageResult<ExecutionResponse>> {
  let res = execution_svc.query_executions(input).await?;
  ok_json!(res)
}

/// 获取执行详情
pub async fn get_execution(
  execution_svc: ExecutionSvc,
  Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionResponse> {
  let res = execution_svc.find_execution_by_id(execution_id).await?;
  ok_json!(res)
}

/// 取消执行
pub async fn cancel_execution(execution_svc: ExecutionSvc, Path(execution_id): Path<ExecutionId>) -> WebResult<()> {
  execution_svc.cancel_execution(execution_id).await?;
  ok_json!()
}

/// 重试执行
pub async fn retry_execution(execution_svc: ExecutionSvc, Path(execution_id): Path<ExecutionId>) -> WebResult<()> {
  execution_svc.retry_execution(execution_id).await?;
  ok_json!()
}

/// 获取执行日志
pub async fn get_execution_logs(
  execution_svc: ExecutionSvc,
  Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionLogResponse> {
  let res = execution_svc.logs(execution_id).await?;
  ok_json!(res)
}

/// 轻量获取执行状态
pub async fn get_execution_status(
  execution_svc: ExecutionSvc,
  Path(execution_id): Path<ExecutionId>,
) -> WebResult<ExecutionStatusResponse> {
  let execution = execution_svc.find_execution_by_id(execution_id).await?;

  let status_response = ExecutionStatusResponse {
    id: execution.id,
    status: execution.status,
    started_at: execution.started_at,
    finished_at: execution.finished_at,
    error: execution.error,
    progress: None, // TODO: 计算执行进度
  };

  ok_json!(status_response)
}

/// 流式订阅执行日志
pub async fn stream_execution_logs(
  execution_svc: ExecutionSvc,
  Path(execution_id): Path<ExecutionId>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
  let stream = async_stream::stream! {
    // 首先检查执行是否存在
    match execution_svc.find_execution_by_id(execution_id).await {
      Ok(execution) => {
        // 发送初始状态
        let initial_event = Event::default()
          .data(format!("Execution {} status: {:?}", execution_id, execution.status));
        yield Ok(initial_event);

        // 如果执行已完成，发送历史日志并结束
        if matches!(execution.status, ExecutionStatus::Success | ExecutionStatus::Failed | ExecutionStatus::Cancelled | ExecutionStatus::Crashed) {
          if let Ok(logs) = execution_svc.logs(execution_id).await {
            for (i, log) in logs.into_iter().enumerate() {
              let log_event = Event::default()
                .event("log")
                .id(format!("log-{}", i))
                .data(serde_json::to_string(&log).unwrap_or_default());
              yield Ok(log_event);
            }
          }

          // 发送完成事件
          let complete_event = Event::default()
            .event("complete")
            .data("Stream completed");
          yield Ok(complete_event);
          return;
        }

        // 对于正在进行的执行，定期检查状态和日志
        let mut last_log_count = 0;
        loop {
          tokio::time::sleep(Duration::from_secs(2)).await;

          // 检查执行状态
          match execution_svc.find_execution_by_id(execution_id).await {
            Ok(current_execution) => {
              // 获取新的日志
              if let Ok(logs) = execution_svc.logs(execution_id).await {
                if logs.len() > last_log_count {
                  for (i, log) in logs.iter().skip(last_log_count).enumerate() {
                    let log_event = Event::default()
                      .event("log")
                      .id(format!("log-{}", last_log_count + i))
                      .data(serde_json::to_string(log).unwrap_or_default());
                    yield Ok(log_event);
                  }
                  last_log_count = logs.len();
                }
              }

              // 如果执行完成，发送完成事件并退出
              if matches!(current_execution.status,
                ExecutionStatus::Success | ExecutionStatus::Failed |
                ExecutionStatus::Cancelled | ExecutionStatus::Crashed) {

                let complete_event = Event::default()
                  .event("complete")
                  .data(format!("Execution completed with status: {:?}", current_execution.status));
                yield Ok(complete_event);
                break;
              }
            }
            Err(_) => {
              // 执行不存在或出错，发送错误事件并退出
              let error_event = Event::default()
                .event("error")
                .data("Execution not found or error occurred");
              yield Ok(error_event);
              break;
            }
          }
        }
      }
      Err(_) => {
        // 执行不存在
        let error_event = Event::default()
          .event("error")
          .data("Execution not found");
        let _ = yield Ok(error_event);
      }
    }
  };

  Sse::new(stream)
}
