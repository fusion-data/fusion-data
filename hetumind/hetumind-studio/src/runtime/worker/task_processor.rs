use async_trait::async_trait;
use hetumind_core::{
  task::{QueueTask, TaskResult},
  workflow::{ExecutionStatus, ParameterMap},
};
use modelsql::ModelManager;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use fusion_common::time::now;
use fusion_core::component::Component;

use crate::{infra::db::execution::ExecutionStoreService, runtime::workflow::WorkflowEngineService};

/// 工作流任务负载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTaskPayload {
  /// 输入数据
  pub trigger_data: ParameterMap,
}

#[derive(Debug, Error)]
pub enum ProcessError {
  #[error("Failed to load workflow: {0}")]
  WorkflowLoadError(String),
  #[error("Failed to update execution status: {0}")]
  ExecutionStatusUpdateError(String),
  #[error("Failed to execute workflow: {0}")]
  WorkflowExecutionError(String),
  #[error("Serialization error: {0}")]
  SerializationError(#[from] serde_json::Error),
}

#[async_trait]
pub trait TaskProcessor: Send + Sync {
  async fn process(&self, task: &QueueTask) -> Result<TaskResult, ProcessError>;
}

#[derive(Clone, Component)]
pub struct WorkflowTaskProcessor {
  #[component]
  mm: ModelManager,
  #[component]
  engine: WorkflowEngineService,
  #[component]
  execution_store: ExecutionStoreService,
}

impl WorkflowTaskProcessor {
  async fn update_execution_status(
    &self,
    execution_id: uuid::Uuid,
    status: ExecutionStatus,
  ) -> Result<(), ProcessError> {
    // TODO: 实现执行状态更新逻辑
    Ok(())
  }
}

#[async_trait]
impl TaskProcessor for WorkflowTaskProcessor {
  async fn process(&self, task: &QueueTask) -> Result<TaskResult, ProcessError> {
    let start_time = now();

    // 解析任务负载
    let payload: WorkflowTaskPayload = serde_json::from_value(task.payload.clone())?;

    todo!()
  }
}
