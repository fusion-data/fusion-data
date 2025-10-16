//! 权限常量与枚举（统一跨应用的资源动作定义）
//! 用法：
//! - 资源层权限判定（字符串常量）：permissions::ResourceAction::WorkflowWrite
//! - 服务内部强类型：ResourceAction::WorkflowWrite.as_str()
use strum::AsRefStr;

/* 函数级注释：强类型枚举，服务内部可使用并映射到字符串常量 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AsRefStr)]
pub enum ResourceAction {
  #[strum(serialize = "workflow:read")]
  WorkflowRead,
  #[strum(serialize = "workflow:write")]
  WorkflowWrite,
  #[strum(serialize = "execution:execute")]
  ExecutionExecute,
  #[strum(serialize = "execution:cancel")]
  ExecutionCancel,
  #[strum(serialize = "execution:retry")]
  ExecutionRetry,
}

impl ResourceAction {
  pub fn as_str(&self) -> &str {
    self.as_ref()
  }
}
