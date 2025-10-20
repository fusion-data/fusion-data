use fusionsql::{ModelManager, SqlError, base::DbBmc};
use serde_json::Value;
use std::time::Duration;

/// 表名常量
pub const TABLE_RESOURCE_MAPPING_CACHE: &str = "resource_mapping_cache";

pub struct ResourceMappingCacheBmc;

impl DbBmc for ResourceMappingCacheBmc {
  const TABLE: &'static str = TABLE_RESOURCE_MAPPING_CACHE;
}

impl ResourceMappingCacheBmc {
  /// 获取缓存
  pub async fn get(mm: &ModelManager, cache_key: &str) -> Result<Option<Value>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let rows = db
      .fetch_all(
        sqlx::query_as::<_, (String,)>(
          "SELECT mapping_response FROM resource_mapping_cache WHERE cache_key = $1 AND expires_at > NOW()",
        )
        .bind(cache_key),
      )
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Query error: {}", e) })?;

    if let Some((value_str,)) = rows.first() { Ok(Some(serde_json::from_str(value_str)?)) } else { Ok(None) }
  }

  /// 设置缓存
  pub async fn set(
    mm: &ModelManager,
    cache_key: &str,
    service: &str,
    _path: &str,
    _method: &str,
    value: &Value,
    ttl: Duration,
  ) -> Result<(), SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      INSERT INTO resource_mapping_cache
      (cache_key, service, mapping_response, expires_at)
      VALUES ($1, $2, $3, NOW() + $4::INTERVAL)
      ON CONFLICT (cache_key)
      DO UPDATE SET
        mapping_response = EXCLUDED.mapping_response,
        expires_at = EXCLUDED.expires_at
    "#;

    db.execute(
      sqlx::query(sql)
        .bind(cache_key)
        .bind(service)
        .bind(
          serde_json::to_string(value)
            .map_err(|e| SqlError::InvalidArgument { message: format!("JSON serialization error: {}", e) })?,
        )
        .bind(format!("{} seconds", ttl.as_secs())),
    )
    .await
    .map_err(|e| SqlError::InvalidArgument { message: format!("Insert error: {}", e) })?;

    Ok(())
  }

  /// 清除过期缓存
  pub async fn cleanup_expired(mm: &ModelManager) -> Result<u64, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let result = db
      .execute(sqlx::query("DELETE FROM resource_mapping_cache WHERE expires_at <= NOW()"))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Delete error: {}", e) })?;

    Ok(result)
  }

  /// 清除服务缓存
  pub async fn clear_service_cache(mm: &ModelManager, service: &str) -> Result<u64, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let result = db
      .execute(sqlx::query("DELETE FROM resource_mapping_cache WHERE service = $1").bind(service))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Delete error: {}", e) })?;

    Ok(result)
  }

  /// 批量清除缓存（根据模式）
  pub async fn clear_by_pattern(mm: &ModelManager, pattern: &str) -> Result<u64, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let result = db
      .execute(sqlx::query("DELETE FROM resource_mapping_cache WHERE cache_key LIKE $1").bind(pattern))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Delete error: {}", e) })?;

    Ok(result)
  }

  /// 获取缓存统计信息
  pub async fn get_cache_stats(mm: &ModelManager) -> Result<CacheStats, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let row = db
      .fetch_one(sqlx::query_as::<_, CacheStats>(
        r#"
          SELECT
            COUNT(*) as total_entries,
            COUNT(*) FILTER (WHERE expires_at <= NOW()) as expired_entries,
            COUNT(*) FILTER (WHERE expires_at > NOW()) as valid_entries,
            COUNT(DISTINCT service) as unique_services
          FROM resource_mapping_cache
        "#,
      ))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Query error: {}", e) })?;

    Ok(row)
  }
}

/// 缓存统计信息
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct CacheStats {
  pub total_entries: i64,
  pub expired_entries: i64,
  pub valid_entries: i64,
  pub unique_services: i64,
}
