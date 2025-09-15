use chrono::{DateTime, FixedOffset};
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{JobStatus, Labels, ResourceLimits};

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
  /// 命令。如： python, uv/uvx, npx, node, bash, sh, cargo, rustc 等
  pub cmd: String,
  /// 命令参数
  pub args: Vec<String>,
  /// 工作目录，不设置则使用默认值
  pub working_directory: Option<String>,
  /// 是否捕获输出
  pub capture_output: bool,
  /// 最大输出大小(字节)
  pub max_output_size: u64,
  /// 任务标签。可用于限制哪些 Agent 允许执行该任务
  pub labels: Labels,
  /// 资源限制
  pub resource_limits: Option<ResourceLimits>,
}

/// SchedJob 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields, sqlx::FromRow), sea_query::enum_def(table_name = "sched_job"))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedJob {
  pub id: Uuid,
  pub namespace_id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: TaskConfig,
  pub status: JobStatus,
  pub created_by: i64,
  pub created_at: DateTime<FixedOffset>,
  pub updated_by: Option<i64>,
  pub updated_at: Option<DateTime<FixedOffset>>,
}

/// Job 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobForCreate {
  pub id: Option<Uuid>,
  pub namespace_id: Option<Uuid>,
  pub name: String,
  pub description: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: Option<serde_json::Value>,
  pub status: JobStatus,
}

/// Job 更新模型
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobForUpdate {
  pub namespace_id: Option<Uuid>,
  pub name: Option<String>,
  pub description: Option<String>,
  pub command: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: Option<serde_json::Value>,
  pub status: Option<JobStatus>,
  pub update_mask: Option<FieldMask>,
}

/// Job 查询请求
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobForQuery {
  pub filter: JobFilter,
  pub page: Page,
}

/// Job 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct JobFilter {
  pub id: Option<OpValsUuid>,
  pub name: Option<OpValsString>,
  pub namespace_id: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}
