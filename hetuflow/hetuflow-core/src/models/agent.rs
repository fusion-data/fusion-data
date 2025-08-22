use std::collections::HashMap;

use garde::Validate;
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use ultimate_common::time::OffsetDateTime;
use uuid::Uuid;

use crate::types::AgentStatus;

/// Agent 性能指标
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentCapabilities {
  /// 最大并发任务数
  pub max_concurrent_tasks: u32,
  /// 资源描述 (cpu, memory, etc.)
  pub resources: HashMap<String, String>,
  /// 支持的特性列表
  pub features: Vec<String>,
  /// Agent 标签，用于筛选任务。比如某些需要特定资源的任务只能在匹配标签的 Agent 上运行
  pub tags: HashMap<String, Option<Box<serde_json::Value>>>,
}

/// AgentEntity 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(sqlx::FromRow, modelsql::field::Fields),
  sea_query::enum_def(table_name = "sched_agent")
)]
pub struct AgentEntity {
  pub id: Uuid,
  pub description: Option<String>,
  pub server_id: Uuid,
  pub host: String,
  pub port: i32,
  pub status: AgentStatus,
  pub capabilities: AgentCapabilities,
  pub last_heartbeat: OffsetDateTime,
  pub created_by: i64,
  pub created_at: OffsetDateTime,
  pub updated_by: Option<i64>,
  pub updated_at: Option<OffsetDateTime>,
}

/// Agent 创建模型
#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
pub struct AgentForCreate {
  #[garde(skip)]
  pub id: Uuid,
  #[garde(skip)]
  pub server_id: Uuid,
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
pub struct AgentForUpdate {
  pub server_id: Option<Uuid>,
  pub description: Option<String>,
  pub host: Option<String>,
  pub port: Option<i32>,
  pub status: Option<AgentStatus>,
  pub capabilities: Option<AgentCapabilities>,
  pub last_heartbeat: Option<OffsetDateTime>,
  pub update_mask: Option<FieldMask>,
}

/// Agent 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
pub struct AgentFilter {
  pub id: Option<OpValsUuid>,
  pub server_id: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub host: Option<OpValsString>,
  pub port: Option<OpValsInt32>,
  pub last_heartbeat: Option<OpValsDateTime>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}

/// Agent 查询请求
#[derive(Default, Deserialize)]
pub struct AgentForQuery {
  pub filter: AgentFilter,
  pub page: Page,
}

#[cfg(feature = "with-db")]
mod with_db {
  use sqlx::encode::IsNull;
  use sqlx::error::BoxDynError;
  use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
  use sqlx::types::Json;
  use sqlx::{Decode, Encode, Postgres, Type};

  use super::*;

  impl From<AgentCapabilities> for sea_query::Value {
    fn from(value: AgentCapabilities) -> Self {
      sea_query::Value::Json(Some(Box::new(serde_json::to_value(value).unwrap())))
    }
  }
  impl sea_query::Nullable for AgentCapabilities {
    fn null() -> sea_query::Value {
      sea_query::Value::Json(None)
    }
  }
  impl Type<Postgres> for AgentCapabilities {
    fn type_info() -> PgTypeInfo {
      <Json<Self> as Type<Postgres>>::type_info()
    }
  }
  impl PgHasArrayType for AgentCapabilities {
    fn array_type_info() -> PgTypeInfo {
      <Json<Self> as PgHasArrayType>::array_type_info()
    }
  }
  impl Encode<'_, Postgres> for AgentCapabilities {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
      Json(self).encode_by_ref(buf)
    }
  }
  impl<'r> Decode<'r, Postgres> for AgentCapabilities {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
      Ok(Json::<Self>::decode(value)?.0)
    }
  }
}
