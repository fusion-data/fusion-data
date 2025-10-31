//! Manual Trigger Node 参数配置
//!
//! 定义手动触发器的配置参数和数据结构

use hetumind_core::types::JsonValue;
use serde::{Deserialize, Serialize};

/// Manual Trigger 配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTriggerConfig {
  /// 执行模式
  pub execution_mode: ExecutionMode,
  /// 是否启用
  pub enabled: bool,
}

/// 执行模式枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
  /// 测试模式
  Test,
  /// 生产模式
  Production,
}

impl Default for ManualTriggerConfig {
  fn default() -> Self {
    Self { execution_mode: ExecutionMode::Test, enabled: true }
  }
}

impl ManualTriggerConfig {
  /// 生成触发数据
  pub fn generate_trigger_data(&self) -> JsonValue {
    serde_json::json!({
        "trigger_type": "manual",
        "execution_mode": match self.execution_mode {
            ExecutionMode::Test => "test",
            ExecutionMode::Production => "production",
        },
        "timestamp": chrono::Utc::now().timestamp(),
        "trigger_id": uuid::Uuid::now_v7().to_string(),
        "message": match self.execution_mode {
            ExecutionMode::Test => "工作流在测试模式下手动触发",
            ExecutionMode::Production => "工作流在生产模式下手动触发",
        },
        "enabled": self.enabled,
    })
  }
}
