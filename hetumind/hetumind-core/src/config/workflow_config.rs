use serde::{Deserialize, Serialize};

use crate::workflow::{ErrorHandlingStrategy, ExecutionMode};

/// 执行配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowConfig {
  /// 错误处理策略
  pub error_handling_strategy: ErrorHandlingStrategy,

  /// 执行模式
  pub execution_mode: ExecutionMode,

  /// 工作流执行总超时时间（毫秒）。当 execution_mode 为 Local 时，此配置有效，其它忽略。
  pub execution_timeout_ms: u64,
}
