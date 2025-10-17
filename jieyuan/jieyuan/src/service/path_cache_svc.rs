use std::time::Duration;

use fusionsql::{ModelManager, SqlError};
use jieyuan_core::model::{PathLookupRequest, PathLookupResponse};

use crate::model::PathCacheBmc;

/// 路径缓存服务 - 便利性包装器
/// 主要的缓存操作在 PathCacheBmc 中，这个服务提供便利方法
pub struct PathCacheSvc;

impl PathCacheSvc {
  /// 获取路径查找缓存
  pub async fn get(mm: &ModelManager, cache_key: &str) -> Result<Option<PathLookupResponse>, SqlError> {
    if let Some(value) = PathCacheBmc::get(mm, cache_key).await? {
      Ok(Some(serde_json::from_value(value)?))
    } else {
      Ok(None)
    }
  }

  /// 设置路径查找缓存
  pub async fn set_path_lookup(
    mm: &ModelManager,
    cache_key: &str,
    req: &PathLookupRequest,
    response: &PathLookupResponse,
  ) -> Result<(), SqlError> {
    let value = serde_json::to_value(response)?;
    PathCacheBmc::set(
      mm,
      cache_key,
      &req.service,
      &req.path,
      &req.method,
      &value,
      Duration::from_secs(response.cache_ttl.unwrap_or(300)),
    )
    .await
  }

  /// 清除服务缓存
  pub async fn clear_service_cache(mm: &ModelManager, service: &str) -> Result<(), SqlError> {
    let _ = PathCacheBmc::clear_service_cache(mm, service).await?;
    Ok(())
  }

  /// 清理过期缓存
  pub async fn cleanup_expired(mm: &ModelManager) -> Result<u64, SqlError> {
    PathCacheBmc::cleanup_expired(mm).await
  }
}
