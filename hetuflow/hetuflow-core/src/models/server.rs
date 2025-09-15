use fusion_common::time::OffsetDateTime;
use modelsql_core::{
  field::FieldMask,
  filter::{OpValsDateTime, OpValsInt32, OpValsString, OpValsUuid, Page},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::ServerStatus;

/// SchedServer 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
  feature = "with-db",
  derive(modelsql::Fields, sqlx::FromRow),
  sea_query::enum_def(table_name = "sched_server")
)]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
pub struct SchedServer {
  pub id: String,
  pub name: String,
  pub address: String,
  pub bind_namespaces: Vec<Uuid>,
  pub status: ServerStatus,
  pub description: Option<String>,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub last_heartbeat: OffsetDateTime,
  pub created_by: i64,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub created_at: OffsetDateTime,
  #[cfg_attr(feature = "with-openapi", schema(value_type = String, format = DateTime, example = "2023-01-01T00:00:00Z"))]
  pub deleted_at: Option<OffsetDateTime>,
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
  pub bind_namespaces: Option<Vec<Uuid>>,
  pub status: Option<ServerStatus>,
  pub description: Option<String>,
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
  pub bind_namespaces: Option<OpValsUuid>,
  pub status: Option<OpValsInt32>,
  pub address: Option<OpValsString>,
  pub created_at: Option<OpValsDateTime>,
  pub updated_at: Option<OpValsDateTime>,
}
