use serde::{Deserialize, Serialize};
use ultimate_common::ahash::HashMap;

/// Agent 配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentSetting {
  pub heartbeat_interval: u32,           // 心跳间隔(秒)
  pub task_timeout: u32,                 // 任务超时时间(秒)
  pub max_output_size: u64,              // 最大输出大小
  pub settings: HashMap<String, String>, // 其他配置项
}
