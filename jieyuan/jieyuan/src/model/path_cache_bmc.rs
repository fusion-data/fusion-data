use fusionsql::{ModelManager, SqlError, base::DbBmc};
use serde_json::Value;
use std::time::Duration;

/// 表名常量
pub const TABLE_PATH_CACHE: &str = "path_lookup_cache";

pub struct PathCacheBmc;

impl DbBmc for PathCacheBmc {
  const TABLE: &'static str = TABLE_PATH_CACHE;
}

impl PathCacheBmc {
  /// 获取缓存
  pub async fn get(mm: &ModelManager, cache_key: &str) -> Result<Option<Value>, SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let rows = db
      .fetch_all(
        sqlx::query_as::<_, (String,)>(
          "SELECT value FROM path_lookup_cache WHERE cache_key = $1 AND expires_at > NOW()",
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
    path: &str,
    method: &str,
    value: &Value,
    ttl: Duration,
  ) -> Result<(), SqlError> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| SqlError::InvalidArgument { message: format!("Database connection error: {}", e) })?;

    let sql = r#"
      INSERT INTO path_lookup_cache
      (cache_key, service, path, method, value, expires_at)
      VALUES ($1, $2, $3, $4, $5, NOW() + $6::INTERVAL)
      ON CONFLICT (cache_key)
      DO UPDATE SET
        value = EXCLUDED.value,
        expires_at = EXCLUDED.expires_at
    "#;

    db.execute(
      sqlx::query(sql)
        .bind(cache_key)
        .bind(service)
        .bind(path)
        .bind(method)
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
      .execute(sqlx::query("DELETE FROM path_lookup_cache WHERE expires_at <= NOW()"))
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
      .execute(sqlx::query("DELETE FROM path_lookup_cache WHERE service = $1").bind(service))
      .await
      .map_err(|e| SqlError::InvalidArgument { message: format!("Delete error: {}", e) })?;

    Ok(result)
  }
}
