use chrono::{DateTime, FixedOffset};
use fusion_common::ahash::HashMap;
use fusion_common::page::Page;
use fusionsql_core::{
  field::FieldMask,
  filter::{OpValDateTime, OpValInt32, OpValString, OpValUuid, OpValValue},
};
use serde::{Deserialize, Serialize};
use strum::AsRefStr;
use uuid::Uuid;

use crate::types::{Labels, ResourceLimits, ScheduleKind, TaskStatus};

/// Executable program command
#[derive(Debug, Clone, Default, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub enum ExecuteCommand {
  #[default]
  #[strum(serialize = "bash")]
  Bash,
  #[strum(serialize = "uv")]
  Uv,
  #[strum(serialize = "python")]
  Python,
  #[strum(serialize = "node")]
  Node,
  #[strum(serialize = "npx")]
  #[serde(rename = "npx")]
  Npx,
  #[strum(serialize = "cargo")]
  Cargo,
  #[strum(serialize = "java")]
  Java,
}

impl ExecuteCommand {
  pub const ALL: &[ExecuteCommand] =
    &[Self::Bash, Self::Uv, Self::Python, Self::Node, Self::Npx, Self::Cargo, Self::Java];

  pub fn is_valid(cmd: &str) -> bool {
    Self::ALL.iter().any(|c| c.as_ref() == cmd)
  }
}

/// 任务配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskConfig {
  /// 超时时间(秒)
  pub timeout: u32,
  /// 最大重试次数
  pub max_retries: u32,
  /// 重试间隔(秒)
  pub retry_interval: u32,
  /// Executable program command
  pub cmd: ExecuteCommand,
  /// 命令参数
  pub args: Vec<String>,
  /// 是否捕获输出
  pub capture_output: bool,
  /// 最大输出大小(字节)
  pub max_output_size: u64,
  /// 任务标签。可用于限制哪些 Agent 允许执行该任务
  pub labels: Option<Labels>,
  /// 资源限制
  pub resource_limits: Option<ResourceLimits>,
}

impl TaskConfig {
  pub fn labels(&self) -> Labels {
    self.labels.clone().unwrap_or_default()
  }
}

/// 任务执行指标
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "with-wasm", derive(tsify::Tsify), tsify(into_wasm_abi, from_wasm_abi))]
pub struct TaskMetrics {
  pub cpu_time: f64,    // CPU 时间
  pub memory_peak: u64, // 内存峰值
  pub disk_read: u64,   // 磁盘读取量
  pub disk_write: u64,  // 磁盘写入量
  pub network_in: u64,  // 网络接收量
  pub network_out: u64, // 网络发送量
}

/// SchedTask 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields, sqlx::FromRow))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedTask {
  pub id: Uuid,
  pub job_id: Uuid,
  pub namespace_id: String,
  /// 任务优先级，数值越大优先级越高
  pub priority: i32,
  pub status: TaskStatus,

  pub schedule_id: Option<Uuid>,
  /// 下一次调度时间。在生成任务时将根据此 调度时间 + schedule_id 判断任务是否已生成，若任务已生成则不会再次生成。
  pub scheduled_at: DateTime<FixedOffset>,
  pub schedule_kind: ScheduleKind,

  /// 任务完成时间。当次任务完成或者所有 Schedule 的配置均已到期
  pub completed_at: Option<DateTime<FixedOffset>>,

  /// 任务环境变量，可能来自 SchedJob 或由事件/手动触发执行传入
  pub environment: Option<serde_json::Value>,

  /// 任务参数，需要为 JSON Object。对于 Event 触发类型的任务，参数为 Event 触发时传入的参数
  pub parameters: serde_json::Value,

  /// 保存 SchedJob.config。当 SchedJob 被修改后，因 SchedTask 保存了 config，所有任务受 SchedJob.config 变更的影响
  pub config: TaskConfig,

  /// 任务重试次数
  pub retry_count: i32,

  pub dependencies: Option<serde_json::Value>,
  pub locked_at: Option<DateTime<FixedOffset>>,
  pub lock_version: i32,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}

impl SchedTask {
  pub fn environments(&self) -> HashMap<String, String> {
    let mut map = HashMap::default();
    if let Some(value) = self.environment.as_ref()
      && let serde_json::Value::Object(env) = value
    {
      for (k, v) in env {
        if let serde_json::Value::String(s) = v {
          map.insert(k.clone(), s.to_string());
        } else if let serde_json::Value::Number(n) = v {
          map.insert(k.clone(), n.to_string());
        }
      }
    }

    map
  }
}

/// SchedTask 创建模型
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskForCreate {
  pub id: Option<Uuid>,
  pub job_id: Uuid,
  pub namespace_id: Option<String>,
  pub status: Option<TaskStatus>,
  #[serde(default = "Default::default")]
  pub priority: i32,

  /// The associated Schedule ID, if None, indicates a task created through an event or manually triggered
  pub schedule_id: Option<Uuid>,

  /// The expected scheduling execution time, if None, indicates that the task was created through an event or
  /// manually triggered, and will be executed as soon as possible
  pub scheduled_at: Option<DateTime<FixedOffset>>,

  pub schedule_kind: Option<ScheduleKind>,
  #[serde(default = "default_parameters")]
  pub parameters: serde_json::Value,
  pub environment: Option<serde_json::Value>,
  #[serde(default = "Default::default")]
  pub retry_count: i32,
  pub dependencies: Option<serde_json::Value>,

  /// Task configuration, obtained from SchedJob
  #[serde(skip)]
  pub config: Option<TaskConfig>,
}

fn default_parameters() -> serde_json::Value {
  serde_json::Value::Object(Default::default())
}

/// SchedTask 更新模型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskForUpdate {
  pub priority: Option<i32>,
  pub namespace_id: Option<String>,
  pub status: Option<TaskStatus>,
  pub scheduled_at: Option<DateTime<FixedOffset>>,
  pub completed_at: Option<DateTime<FixedOffset>>,
  pub parameters: Option<serde_json::Value>,
  pub environment: Option<serde_json::Value>,
  pub config: Option<TaskConfig>,
  pub retry_count: Option<i32>,
  pub max_retries: Option<i32>,
  pub dependencies: Option<serde_json::Value>,
  pub locked_at: Option<DateTime<FixedOffset>>,
  pub lock_version: Option<i32>,
  #[serde(skip)]
  pub update_mask: Option<FieldMask>,
}

/// SchedTask 查询请求
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskForQuery {
  pub filter: TaskFilter,
  pub page: Page,
}

/// SchedTask 过滤器
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskFilter {
  pub id: Option<OpValUuid>,
  pub job_id: Option<OpValUuid>,
  pub schedule_id: Option<OpValUuid>,
  pub namespace_id: Option<OpValString>,
  pub task_config: Option<OpValValue>,
  pub status: Option<OpValInt32>,
  pub scheduled_at: Option<OpValDateTime>,
  pub locked_at: Option<OpValDateTime>,
  pub created_at: Option<OpValDateTime>,
  pub updated_at: Option<OpValDateTime>,
}
