//! 权限常量与枚举（统一跨应用的资源动作定义）
//! 用法：
//! - 资源层权限判定（字符串常量）：permissions::WORKFLOW_WRITE
//! - 服务内部强类型：ResourceAction::WorkflowWrite.as_str()

/* 函数级注释：权限常量，便于与令牌 claims/scopes 对齐 */
pub const WORKFLOW_READ: &str = "workflow:read";
pub const WORKFLOW_WRITE: &str = "workflow:write";
pub const EXECUTION_EXECUTE: &str = "execution:execute";
pub const EXECUTION_CANCEL: &str = "execution:cancel";
pub const EXECUTION_RETRY: &str = "execution:retry";

/* 函数级注释：强类型枚举，服务内部可使用并映射到字符串常量 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceAction {
  WorkflowRead,
  WorkflowWrite,
  ExecutionExecute,
  ExecutionCancel,
  ExecutionRetry,
}

impl ResourceAction {
  pub fn as_str(&self) -> &'static str {
    match self {
      ResourceAction::WorkflowRead => WORKFLOW_READ,
      ResourceAction::WorkflowWrite => WORKFLOW_WRITE,
      ResourceAction::ExecutionExecute => EXECUTION_EXECUTE,
      ResourceAction::ExecutionCancel => EXECUTION_CANCEL,
      ResourceAction::ExecutionRetry => EXECUTION_RETRY,
    }
  }

  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      WORKFLOW_READ => Some(ResourceAction::WorkflowRead),
      WORKFLOW_WRITE => Some(ResourceAction::WorkflowWrite),
      EXECUTION_EXECUTE => Some(ResourceAction::ExecutionExecute),
      EXECUTION_CANCEL => Some(ResourceAction::ExecutionCancel),
      EXECUTION_RETRY => Some(ResourceAction::ExecutionRetry),
      _ => None,
    }
  }
}