use async_trait::async_trait;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::workflow::WorkflowId;

use super::{ExecutionData, NodeExecutionContext, TriggerError, WorkflowNode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum TriggerKind {
  /// 手动触发
  Manual = 1,
  /// Webhook 触发
  Webhook = 2,
  /// 定时触发
  Schedule = 3,
  /// 轮询触发
  Poll = 4,
  /// 文件监听
  FileWatch = 5,
}

pub struct TriggerContext {
  pub node_context: NodeExecutionContext,
}

#[async_trait]
pub trait TriggerController {
  /// 等待触发事件
  async fn wait_for_trigger(&mut self) -> Result<Vec<ExecutionData>, TriggerError>;

  /// 是否仍然活跃
  fn is_active(&self) -> bool;

  /// 停止触发器
  async fn stop(&mut self) -> Result<(), TriggerError>;
}

pub type TriggerHandle = Box<dyn TriggerController + Send + Sync>;

#[async_trait]
pub trait TriggerExecutor: Send + Sync {
  /// 启动触发器
  async fn start_trigger(
    &self,
    workflow_id: WorkflowId,
    node: &WorkflowNode,
    context: &TriggerContext,
  ) -> Result<TriggerHandle, TriggerError>;

  /// 停止触发器
  async fn stop_trigger(&self, handle: &TriggerHandle) -> Result<(), TriggerError>;

  /// 触发器类型
  fn trigger_type(&self) -> TriggerKind;
}
