use chrono::{DateTime, FixedOffset};
use fusion_common::page::Page;
use fusionsql_core::{
  field::FieldMask,
  filter::{OpValDateTime, OpValInt32, OpValString},
};
use serde::{Deserialize, Serialize};

use crate::types::ServerStatus;

/// SchedServer 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields, sqlx::FromRow))]
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
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerForRegister {
  pub id: String,
  pub name: String,
  pub address: String,
  pub status: ServerStatus,
}

/// Server 更新模型
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::Fields))]
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
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerForQuery {
  pub filter: ServerFilter,
  pub page: Page,
}

/// Server 过滤器
#[derive(Default, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(fusionsql::FilterNodes))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct ServerFilter {
  pub id: Option<OpValString>,
  pub name: Option<OpValString>,
  pub bind_namespaces: Option<OpValString>,
  pub status: Option<OpValInt32>,
  pub address: Option<OpValString>,
  pub created_at: Option<OpValDateTime>,
  pub last_heartbeat_at: Option<OpValDateTime>,
}
