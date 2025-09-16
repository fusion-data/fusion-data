use chrono::{DateTime, FixedOffset};
use modelsql::filter::OpValsValue;
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{ScheduleKind, TaskStatus};

use super::TaskConfig;

/// 任务执行指标
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TaskMetrics {
  pub start_time: i64,       // 开始时间
  pub end_time: Option<i64>, // 结束时间
  pub cpu_time: f64,         // CPU 时间
  pub memory_peak: u64,      // 内存峰值
  pub disk_read: u64,        // 磁盘读取量
  pub disk_write: u64,       // 磁盘写入量
  pub network_in: u64,       // 网络接收量
  pub network_out: u64,      // 网络发送量
}

/// SchedTask 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(modelsql::Fields, sqlx::FromRow),
  sea_query::enum_def(table_name = "sched_task")
)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedTask {
  pub id: Uuid,
  pub job_id: Uuid,
  pub namespace_id: Uuid,
  /// 任务优先级，数值越大优先级越高
  pub priority: i32,
  pub status: TaskStatus,

  pub schedule_id: Option<Uuid>,
  /// 下一次调度时间。在生成任务时将根据此 调度时间 + schedule_id 判断任务是否已生成，若任务已生成则不会再次生成。
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub scheduled_at: DateTime<FixedOffset>,
  pub schedule_kind: ScheduleKind,

  /// 任务完成时间。当次任务完成或者所有 Schedule 的配置均已到期
  #[cfg_attr(feature = "with-openapi", schema(value_type = Option<String>, format = DateTime, example = "2023-01-01T00:00:00Z"))]
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
  #[cfg_attr(feature = "with-openapi", schema(value_type = Option<String>, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub locked_at: Option<DateTime<FixedOffset>>,
  pub lock_version: i32,
  pub created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = Option<String>, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub updated_at: Option<DateTime<FixedOffset>>,
}

/// SchedTask 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskForCreate {
  pub id: Uuid,
  pub job_id: Uuid,
  pub namespace_id: Uuid,
  pub status: TaskStatus,
  pub priority: i32,
  /// 关联的 Schedule ID，若为 None 则表示为通过事件或手动触发创建的任务
  pub schedule_id: Option<Uuid>,
  pub scheduled_at: DateTime<FixedOffset>,
  pub parameters: serde_json::Value,
  pub environment: Option<serde_json::Value>,
  pub config: Option<TaskConfig>,
  pub retry_count: i32,
  pub dependencies: Option<serde_json::Value>,
}

/// SchedTask 更新模型
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskForUpdate {
  pub priority: Option<i32>,
  pub namespace_id: Option<Uuid>,
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
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskForQuery {
  pub filter: TaskFilter,
  pub page: Page,
}

/// SchedTask 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct TaskFilter {
  pub id: Option<OpValsUuid>,
  pub job_id: Option<OpValsUuid>,
  pub schedule_id: Option<OpValsUuid>,
  pub namespace_id: Option<OpValsUuid>,
  pub task_config: Option<OpValsValue>,
  pub status: Option<OpValsInt32>,
  pub scheduled_at: Option<OpValsDateTime>,
  pub locked_at: Option<OpValsDateTime>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}
