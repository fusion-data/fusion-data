//! 二进制数据管理器工厂
use std::sync::Arc;

use hetumind_core::binary_storage::{BinaryDataManager, BinaryStorageConfig, BinaryStorageError, StorageKind};
use log::info;

use crate::binary_storage::{create_fs_storage, create_memory_storage, create_s3_storage};

/// 二进制数据管理器工厂
///
/// 提供创建和配置二进制数据管理器的便捷方法。
pub struct BinaryDataManagerFactory;

impl BinaryDataManagerFactory {
  /// 创建基于配置的二进制数据管理器
  ///
  /// # 参数
  /// - `config`: 存储配置
  ///
  /// # 返回
  /// - `Result<BinaryDataManager, BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_from_config(config: BinaryStorageConfig) -> Result<BinaryDataManager, BinaryStorageError> {
    // 创建存储实现
    let storage = match config.kind {
      StorageKind::Fs => {
        let root = config.root.as_str();
        info!("创建文件系统存储: root={}", root);
        Arc::new(create_fs_storage(root).await?)
      }
      StorageKind::S3 => {
        let bucket = config.bucket.as_deref().ok_or_else(|| BinaryStorageError::config("缺少 bucket 参数"))?;
        let region = config.region.as_deref().ok_or_else(|| BinaryStorageError::config("缺少 region 参数"))?;
        let access_key =
          config.access_key.as_deref().ok_or_else(|| BinaryStorageError::config("缺少 access_key 参数"))?;
        let secret_key =
          config.secret_key.as_deref().ok_or_else(|| BinaryStorageError::config("缺少 secret_key 参数"))?;
        let endpoint = config.endpoint.as_deref();
        info!("创建S3存储: bucket={}, region={}", bucket, region);
        Arc::new(create_s3_storage(bucket, region, access_key, secret_key, endpoint).await?)
      }
      StorageKind::Memory => {
        info!("创建内存存储");
        Arc::new(create_memory_storage().await?)
      }
    };

    // 创建数据管理器
    let cache_limit = config.cache_limit.unwrap_or(200 * 1024 * 1024); // 默认200MB
    let data_manager = BinaryDataManager::new(storage, cache_limit)?;

    Ok(data_manager)
  }

  /// 创建文件系统存储的管理器
  ///
  /// # 参数
  /// - `root`: 根目录路径
  /// - `cache_limit`: 缓存大小限制（字节）
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_fs_manager(
    root: &str,
    cache_limit: Option<usize>,
  ) -> Result<BinaryDataManager, BinaryStorageError> {
    let mut config = BinaryStorageConfig::fs(root);
    config.cache_limit = cache_limit;
    Self::create_from_config(config).await
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
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_s3_manager(
    bucket: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    endpoint: Option<&str>,
    cache_limit: Option<usize>,
  ) -> Result<BinaryDataManager, BinaryStorageError> {
    let config = BinaryStorageConfig {
      kind: StorageKind::S3,
      root: "s3".to_string(),
      bucket: Some(bucket.to_string()),
      region: Some(region.to_string()),
      access_key: Some(access_key.to_string()),
      secret_key: Some(secret_key.to_string()),
      endpoint: endpoint.map(|s| s.to_string()),
      cache_limit,
    };
    Self::create_from_config(config).await
  }

  /// 创建内存存储的管理器
  ///
  /// # 参数
  /// - `cache_limit`: 缓存大小限制（字节）
  ///
  /// # 返回
  /// - `Result<Arc<BinaryDataLifecycleManager>, BinaryStorageError>`: 创建的生命周期管理器
  pub async fn create_memory_manager(cache_limit: Option<usize>) -> Result<BinaryDataManager, BinaryStorageError> {
    let config = BinaryStorageConfig::memory(cache_limit);
    Self::create_from_config(config).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hetumind_core::binary_storage::BinaryDataMetadata;

  #[tokio::test]
  async fn test_create_memory_manager() {
    let manager = BinaryDataManagerFactory::create_memory_manager(None).await.unwrap();

    // 测试存储数据
    let data = b"test data".to_vec();
    let metadata = BinaryDataMetadata::default();

    let data_ref = manager.store_data(data.clone(), metadata).await.unwrap();
    let retrieved = manager.get_data(&data_ref.file_key).await.unwrap();

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

    let manager = BinaryDataManagerFactory::create_fs_manager(&test_dir_str, None).await.unwrap();

    // 测试存储数据
    let data = b"test data".to_vec();
    let metadata = BinaryDataMetadata::default();

    let data_ref = manager.store_data(data.clone(), metadata).await.unwrap();
    let retrieved = manager.get_data(&data_ref.file_key).await.unwrap();

    assert_eq!(retrieved, data);

    // 清理
    std::fs::remove_dir_all(&test_dir).ok();
  }
}
