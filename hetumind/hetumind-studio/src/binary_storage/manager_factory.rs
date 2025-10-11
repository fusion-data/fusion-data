//! 二进制数据管理器工厂
use std::sync::Arc;

use hetumind_core::binary_storage::{
  BinaryDataLifecycleManager, BinaryDataManager, BinaryStorageConfig, BinaryStorageError, LifecycleCleanupConfig,
  StorageType,
};
use log::{info, warn};

use crate::binary_storage::{create_fs_storage, create_memory_storage, create_redis_storage, create_s3_storage};

/// 二进制数据管理器工厂
///
/// 提供创建和配置二进制数据管理器的便捷方法。
pub struct BinaryDataManagerFactory;

impl BinaryDataManagerFactory {
  /// 创建基于配置的二进制数据管理器
  ///
  /// # 参数
  /// - `config`: 存储配置
  /// - `cache_limit`: 缓存大小限制（字节）
  /// - `lifecycle_config`: 生命周期配置
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_from_config(
    config: BinaryStorageConfig,
    cache_limit: Option<usize>,
    lifecycle_config: Option<LifecycleCleanupConfig>,
  ) -> Result<Arc<BinaryDataLifecycleManager>, BinaryStorageError> {
    // 创建存储实现
    let storage = match config.storage_type {
      StorageType::Fs => {
        let root = config.root.as_str();
        info!("创建文件系统存储: root={}", root);
        Arc::new(create_fs_storage(root).await?)
      }
      StorageType::S3 => {
        let bucket = config
          .config
          .get("bucket")
          .and_then(|v| v.as_str())
          .ok_or_else(|| hetumind_core::binary_storage::BinaryStorageError::config("缺少 bucket 参数"))?;
        let region = config
          .config
          .get("region")
          .and_then(|v| v.as_str())
          .ok_or_else(|| hetumind_core::binary_storage::BinaryStorageError::config("缺少 region 参数"))?;
        let access_key = config
          .config
          .get("access_key")
          .and_then(|v| v.as_str())
          .ok_or_else(|| hetumind_core::binary_storage::BinaryStorageError::config("缺少 access_key 参数"))?;
        let secret_key = config
          .config
          .get("secret_key")
          .and_then(|v| v.as_str())
          .ok_or_else(|| hetumind_core::binary_storage::BinaryStorageError::config("缺少 secret_key 参数"))?;
        let endpoint = config.config.get("endpoint").and_then(|v| v.as_str());
        info!("创建S3存储: bucket={}, region={}", bucket, region);
        Arc::new(create_s3_storage(bucket, region, access_key, secret_key, endpoint).await?)
      }
      StorageType::Memory => {
        info!("创建内存存储");
        Arc::new(create_memory_storage().await?)
      }
      StorageType::Redis => {
        let endpoint = config
          .config
          .get("endpoint")
          .and_then(|v| v.as_str())
          .ok_or_else(|| hetumind_core::binary_storage::BinaryStorageError::config("缺少 endpoint 参数"))?;
        let db = config.config.get("db").and_then(|v| v.as_i64()).unwrap_or(0);
        info!("创建Redis存储: endpoint={}, db={}", endpoint, db);
        Arc::new(create_redis_storage(endpoint, db).await?)
      }
    };

    // 创建数据管理器
    let cache_limit = cache_limit.unwrap_or(100 * 1024 * 1024); // 默认100MB
    let data_manager = Arc::new(BinaryDataManager::new(storage, cache_limit)?);

    // 创建生命周期管理器
    let lifecycle_config = lifecycle_config.unwrap_or_default();
    let lifecycle_manager = Arc::new(BinaryDataLifecycleManager::new(data_manager, lifecycle_config));

    // 启动自动清理任务
    lifecycle_manager.start_cleanup_task().await?;

    Ok(lifecycle_manager)
  }

  /// 创建文件系统存储的管理器
  ///
  /// # 参数
  /// - `root`: 根目录路径
  /// - `cache_limit`: 缓存大小限制（字节）
  /// - `lifecycle_config`: 生命周期配置
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_fs_manager(
    root: &str,
    cache_limit: Option<usize>,
    lifecycle_config: Option<LifecycleCleanupConfig>,
  ) -> Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError> {
    let config = BinaryStorageConfig::fs(root);
    Self::create_from_config(config, cache_limit, lifecycle_config).await
  }

  /// 创建S3存储的管理器
  ///
  /// # 参数
  /// - `bucket`: S3桶名
  /// - `region`: AWS区域
  /// - `access_key`: 访问密钥
  /// - `secret_key`: 秘密密钥
  /// - `endpoint`: 自定义端点（可选）
  /// - `cache_limit`: 缓存大小限制（字节）
  /// - `lifecycle_config`: 生命周期配置
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_s3_manager(
    bucket: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    endpoint: Option<&str>,
    cache_limit: Option<usize>,
    lifecycle_config: Option<LifecycleCleanupConfig>,
  ) -> Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError> {
    let config = BinaryStorageConfig::s3(bucket, region, access_key, secret_key)
      .with_extra_config("endpoint", hetumind_core::types::JsonValue::String(endpoint.unwrap_or("").to_string()));

    Self::create_from_config(config, cache_limit, lifecycle_config).await
  }

  /// 创建内存存储的管理器
  ///
  /// # 参数
  /// - `cache_limit`: 缓存大小限制（字节）
  /// - `lifecycle_config`: 生命周期配置
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_memory_manager(
    cache_limit: Option<usize>,
    lifecycle_config: Option<LifecycleCleanupConfig>,
  ) -> Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError> {
    let config = BinaryStorageConfig::memory();
    Self::create_from_config(config, cache_limit, lifecycle_config).await
  }

  /// 创建Redis存储的管理器
  ///
  /// # 参数
  /// - `endpoint`: Redis端点
  /// - `db`: 数据库编号
  /// - `cache_limit`: 缓存大小限制（字节）
  /// - `lifecycle_config`: 生命周期配置
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_redis_manager(
    endpoint: &str,
    db: i64,
    cache_limit: Option<usize>,
    lifecycle_config: Option<LifecycleCleanupConfig>,
  ) -> Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError> {
    let config = BinaryStorageConfig::redis(endpoint, db);
    Self::create_from_config(config, cache_limit, lifecycle_config).await
  }

  /// 从环境变量创建管理器
  ///
  /// 支持以下环境变量：
  /// - `BINARY_STORAGE_TYPE`: 存储类型 (fs, s3, memory, redis)
  /// - `BINARY_STORAGE_ROOT`: 文件系统根目录 (仅fs类型)
  /// - `BINARY_STORAGE_BUCKET`: S3桶名 (仅s3类型)
  /// - `BINARY_STORAGE_REGION`: S3区域 (仅s3类型)
  /// - `BINARY_STORAGE_ACCESS_KEY`: S3访问密钥 (仅s3类型)
  /// - `BINARY_STORAGE_SECRET_KEY`: S3秘密密钥 (仅s3类型)
  /// - `BINARY_STORAGE_ENDPOINT`: S3端点 (可选，仅s3类型)
  /// - `BINARY_STORAGE_REDIS_ENDPOINT`: Redis端点 (仅redis类型)
  /// - `BINARY_STORAGE_REDIS_DB`: Redis数据库编号 (可选，仅redis类型)
  /// - `BINARY_STORAGE_CACHE_LIMIT`: 缓存大小限制（字节）
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_from_env()
  -> Result<Arc<BinaryDataLifecycleManager>, hetumind_core::binary_storage::BinaryStorageError> {
    use std::env;

    // 获取存储类型
    let storage_type_str = env::var("BINARY_STORAGE_TYPE").unwrap_or_else(|_| "memory".to_string());

    let storage_type = match storage_type_str.as_str() {
      "fs" => StorageType::Fs,
      "s3" => StorageType::S3,
      "memory" => StorageType::Memory,
      "redis" => StorageType::Redis,
      _ => {
        warn!("未知的存储类型: {}, 使用默认的内存存储", storage_type_str);
        StorageType::Memory
      }
    };

    // 获取缓存限制
    let cache_limit = env::var("BINARY_STORAGE_CACHE_LIMIT").ok().and_then(|s| s.parse().ok());

    // 根据存储类型创建配置
    let config = match storage_type {
      StorageType::Fs => {
        let root = env::var("BINARY_STORAGE_ROOT").unwrap_or_else(|_| "/tmp/hetumind-binary".to_string());
        BinaryStorageConfig::fs(root)
      }
      StorageType::S3 => {
        let bucket = env::var("BINARY_STORAGE_BUCKET").unwrap_or_else(|_| "hetumind-binary".to_string());

        let region = env::var("BINARY_STORAGE_REGION").unwrap_or_else(|_| "us-west-2".to_string());

        let access_key = env::var("BINARY_STORAGE_ACCESS_KEY").map_err(|_| {
          hetumind_core::binary_storage::BinaryStorageError::config("缺少 BINARY_STORAGE_ACCESS_KEY 环境变量")
        })?;

        let secret_key = env::var("BINARY_STORAGE_SECRET_KEY").map_err(|_| {
          hetumind_core::binary_storage::BinaryStorageError::config("缺少 BINARY_STORAGE_SECRET_KEY 环境变量")
        })?;

        let endpoint = env::var("BINARY_STORAGE_ENDPOINT").ok();

        let mut config = BinaryStorageConfig::s3(&bucket, &region, &access_key, &secret_key);

        if let Some(ep) = endpoint {
          config = config.with_extra_config("endpoint", hetumind_core::types::JsonValue::String(ep));
        }

        config
      }
      StorageType::Memory => BinaryStorageConfig::memory(),
      StorageType::Redis => {
        let endpoint = env::var("BINARY_STORAGE_REDIS_ENDPOINT").unwrap_or_else(|_| "localhost:6379".to_string());

        let db = env::var("BINARY_STORAGE_REDIS_DB").ok().and_then(|s| s.parse().ok()).unwrap_or(0);

        BinaryStorageConfig::redis(&endpoint, db)
      }
    };

    // 创建生命周期配置
    let lifecycle_config = LifecycleCleanupConfig::default();

    Self::create_from_config(config, cache_limit, Some(lifecycle_config)).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hetumind_core::binary_storage::BinaryDataMetadata;

  #[tokio::test]
  async fn test_create_memory_manager() {
    let manager = BinaryDataManagerFactory::create_memory_manager(None, None).await.unwrap();

    // 测试存储数据
    let data = b"test data".to_vec();
    let metadata = BinaryDataMetadata::default();

    let key = manager.store_data(data.clone(), metadata).await.unwrap();
    let retrieved = manager.get_data(&key).await.unwrap();

    assert_eq!(retrieved, data);
  }

  #[tokio::test]
  async fn test_create_fs_manager() {
    // 使用临时目录
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("hetumind_binary_test");
    let test_dir_str = test_dir.to_string_lossy();

    // 确保目录存在
    std::fs::create_dir_all(&test_dir).ok();

    let manager = BinaryDataManagerFactory::create_fs_manager(&test_dir_str, None, None).await.unwrap();

    // 测试存储数据
    let data = b"test data".to_vec();
    let metadata = BinaryDataMetadata::default();

    let key = manager.store_data(data.clone(), metadata).await.unwrap();
    let retrieved = manager.get_data(&key).await.unwrap();

    assert_eq!(retrieved, data);

    // 清理
    std::fs::remove_dir_all(&test_dir).ok();
  }
}
