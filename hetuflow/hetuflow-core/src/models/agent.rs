use chrono::{DateTime, FixedOffset};
use fusion_common::ahash::HashMap;
use garde::Validate;
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, Page},
};
use serde::{Deserialize, Serialize};

use crate::types::{AgentStatus, Labels};
use crate::utils::defaults;

/// Agent 性能指标
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentMetrics {
  pub cpu_usage: f64,          // CPU 使用率
  pub memory_usage: f64,       // 内存使用率
  pub disk_usage: f64,         // 磁盘使用率
  pub active_tasks: u32,       // 活跃任务数
  pub total_executed: u64,     // 累计执行任务数
  pub success_rate: f64,       // 成功率
  pub avg_execution_time: f64, // 平均执行时间
  pub uptime: u64,             // 运行时间
}

/// Agent 能力描述
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentCapabilities {
  /// 最大并发任务数
  pub max_concurrent_tasks: u32,
  /// Agent 标签，用于筛选任务。比如某些需要特定资源的任务只能在匹配标签的 Agent 上运行
  pub labels: Labels,
  pub metadata: HashMap<String, String>,
}

/// Agent 统计信息
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentStatistics {
  /// 成功任务数
  #[serde(default = "defaults::default_u64")]
  pub success_tasks: u64,
  /// 失败任务数
  #[serde(default = "defaults::default_u64")]
  pub failure_tasks: u64,
  /// 总任务数
  #[serde(default = "defaults::default_u64")]
  pub total_tasks: u64,
  /// 平均响应时间（毫秒）
  #[serde(default = "defaults::default_f64")]
  pub avg_response_ms: f64,
  /// 最后失败时间（毫秒）
  #[serde(default = "defaults::default_i64")]
  pub last_failure_ms: i64,
  /// 连续失败次数
  #[serde(default = "defaults::default_u32")]
  pub consecutive_failures: u32,
}

/// SchedAgent 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(sqlx::FromRow, modelsql::field::Fields),
  sea_query::enum_def(table_name = "sched_agent")
)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedAgent {
  pub id: String,
  pub description: Option<String>,
  pub address: String,
  pub status: AgentStatus,
  pub capabilities: AgentCapabilities,
  pub statistics: AgentStatistics,
  pub last_heartbeat_at: DateTime<FixedOffset>,
  pub created_at: DateTime<FixedOffset>,
}

/// Agent 创建模型
#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentForCreate {
  #[garde(skip)]
  pub id: String,
  #[garde(skip)]
  pub description: Option<String>,
  #[garde(ip)]
  pub host: String,
  #[garde(range(min = 1, max = 65535))]
  pub port: i32,
  #[garde(skip)]
  pub status: AgentStatus,
  #[garde(skip)]
  pub capabilities: AgentCapabilities,
}

/// Agent 更新模型
#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentForUpdate {
  pub description: Option<String>,
  pub host: Option<String>,
  pub port: Option<i32>,
  pub status: Option<AgentStatus>,
  pub capabilities: Option<AgentCapabilities>,
  pub last_heartbeat_at: Option<DateTime<FixedOffset>>,
  #[serde(skip)]
  pub update_mask: Option<FieldMask>,
}

/// Agent 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentFilter {
  pub id: Option<OpValsString>,
  pub status: Option<OpValsInt32>,
  pub address: Option<OpValsString>,
  pub last_heartbeat_at: Option<OpValsDateTime>,
  pub created_at: Option<OpValsDateTime>,
}

/// Agent 查询请求
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct AgentForQuery {
  pub filter: AgentFilter,
  pub page: Page,
}
