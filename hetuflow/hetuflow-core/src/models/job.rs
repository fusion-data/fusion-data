use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use fusion_common::{ahash::HashMap, time::OffsetDateTime};
use uuid::Uuid;

use crate::types::{JobStatus, ResourceLimits};

/// 任务配置
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JobConfig {
  /// 超时时间(秒)
  pub timeout: u32,
  /// 最大重试次数
  pub max_retries: u32,
  /// 重试间隔(秒)
  pub retry_interval: u32,
  /// 工作目录，不设置则使用默认值
  pub working_directory: Option<String>,
  /// 是否捕获输出
  pub capture_output: bool,
  /// 最大输出大小(字节)
  pub max_output_size: u64,
  /// 任务标签。可用于限制哪些 Agent 允许执行该任务
  pub tags: HashMap<String, Option<serde_json::Value>>,
  /// 资源限制
  pub resource_limits: Option<ResourceLimits>,
}

/// JobEntity 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields, sqlx::FromRow), sea_query::enum_def(table_name = "sched_job"))]
pub struct JobEntity {
  pub id: Uuid,
  pub namespace_id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub environment: Option<serde_json::Value>,
  pub config: Option<JobConfig>,
  pub status: JobStatus,
  pub created_by: i64,
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  pub updated_at: Option<OffsetDateTime>,
}

impl JobEntity {
  pub fn tags(&self) -> Vec<String> {
    self.config.as_ref().map(|c| c.tags.keys().cloned().collect()).unwrap_or_default()
  }
}

/// Job 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
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
pub struct JobForQuery {
  pub filter: JobFilter,
  pub page: Page,
}

/// Job 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
pub struct JobFilter {
  pub id: Option<OpValsUuid>,
  pub name: Option<OpValsString>,
  pub namespace_id: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}

#[cfg(feature = "with-db")]
mod with_db {
  use sqlx::encode::IsNull;
  use sqlx::error::BoxDynError;
  use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
  use sqlx::types::Json;
  use sqlx::{Decode, Encode, Postgres, Type};

  use super::*;

  impl From<JobConfig> for sea_query::Value {
    fn from(value: JobConfig) -> Self {
      Self::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
    }
  }
  impl sea_query::Nullable for JobConfig {
    fn null() -> sea_query::Value {
      sea_query::Value::Json(None)
    }
  }
  impl Type<Postgres> for JobConfig {
    fn type_info() -> PgTypeInfo {
      <Json<Self> as Type<Postgres>>::type_info()
    }
  }
  impl PgHasArrayType for JobConfig {
    fn array_type_info() -> PgTypeInfo {
      <Json<Self> as PgHasArrayType>::array_type_info()
    }
  }
  impl Encode<'_, Postgres> for JobConfig {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
      Json(self).encode_by_ref(buf)
    }
  }
  impl<'r> Decode<'r, Postgres> for JobConfig {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
      Ok(Json::<Self>::decode(value)?.0)
    }
  }
}
