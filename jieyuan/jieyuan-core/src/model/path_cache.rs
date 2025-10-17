use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 路径缓存实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-db", derive(sqlx::FromRow, fusionsql::Fields))]
#[cfg_attr(feature = "with-openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub struct PathCacheEntity {
  pub cache_key: String,
  pub service: String,
  pub path: String,
  pub method: String,
  pub value: String, // JSON string
  pub expires_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

/// 表名常量
pub const TABLE_PATH_CACHE: &str = "path_lookup_cache";
