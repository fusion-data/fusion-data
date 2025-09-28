use chrono::{DateTime, FixedOffset};
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, Page},
};
use serde::{Deserialize, Serialize};

use crate::types::ServerStatus;

/// SchedServer 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields, sqlx::FromRow))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedServer {
  pub id: String,
  pub name: String,
  pub address: String,
  pub bind_namespaces: Vec<String>,
  pub status: ServerStatus,
  pub description: Option<String>,
  pub last_heartbeat_at: DateTime<FixedOffset>,
  pub created_at: DateTime<FixedOffset>,
}

/// Server 创建模型
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerForRegister {
  pub id: String,
  pub name: String,
  pub address: String,
  pub status: ServerStatus,
}

/// Server 更新模型
#[derive(Debug, Clone, Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerForUpdate {
  pub name: Option<String>,
  pub address: Option<String>,
  pub bind_namespaces: Option<Vec<String>>,
  pub status: Option<ServerStatus>,
  pub description: Option<String>,
  pub last_heartbeat_at: Option<DateTime<FixedOffset>>,
  pub update_mask: Option<FieldMask>,
}

/// Server 查询请求
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerForQuery {
  pub filter: ServerFilter,
  pub page: Page,
}

/// Server 过滤器
#[derive(Default, Deserialize)]
#[cfg_attr(feature = "with-db", derive(modelsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerFilter {
  pub id: Option<OpValsString>,
  pub name: Option<OpValsString>,
  pub bind_namespaces: Option<OpValsString>,
  pub status: Option<OpValsInt32>,
  pub address: Option<OpValsString>,
  pub created_at: Option<OpValsDateTime>,
  pub last_heartbeat_at: Option<OpValsDateTime>,
}
