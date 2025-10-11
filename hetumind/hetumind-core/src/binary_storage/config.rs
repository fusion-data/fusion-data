//! 二进制数据存储配置

use crate::types::JsonValue;
use serde::{Deserialize, Serialize};

/// 二进制数据存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStorageConfig {
  /// 存储类型
  pub storage_type: StorageType,
  /// 根路径/桶名
  pub root: String,
  /// 存储特定配置
  pub config: JsonValue,
}

/// 支持的存储类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
  /// 本地文件系统
  Fs,
  /// S3兼容存储
  S3,
  /// 内存存储（主要用于测试）
  Memory,
  /// Redis存储
  Redis,
}

impl BinaryStorageConfig {
  /// 创建本地文件系统配置
  pub fn fs(root: impl Into<String>) -> Self {
    Self { storage_type: StorageType::Fs, root: root.into(), config: JsonValue::Object(serde_json::Map::new()) }
  }

  /// 创建S3配置
  pub fn s3(
    bucket: impl Into<String>,
    region: impl Into<String>,
    access_key: impl Into<String>,
    secret_key: impl Into<String>,
  ) -> Self {
    let mut config = serde_json::Map::new();
    config.insert("bucket".to_string(), JsonValue::String(bucket.into()));
    config.insert("region".to_string(), JsonValue::String(region.into()));
    config.insert("access_key".to_string(), JsonValue::String(access_key.into()));
    config.insert("secret_key".to_string(), JsonValue::String(secret_key.into()));

    Self { storage_type: StorageType::S3, root: "s3".to_string(), config: JsonValue::Object(config) }
  }

  /// 创建内存存储配置
  pub fn memory() -> Self {
    Self {
      storage_type: StorageType::Memory,
      root: "memory".to_string(),
      config: JsonValue::Object(serde_json::Map::new()),
    }
  }

  /// 创建Redis配置
  pub fn redis(endpoint: impl Into<String>, db: i64) -> Self {
    let mut config = serde_json::Map::new();
    config.insert("endpoint".to_string(), JsonValue::String(endpoint.into()));
    config.insert("db".to_string(), JsonValue::Number(db.into()));

    Self { storage_type: StorageType::Redis, root: "redis".to_string(), config: JsonValue::Object(config) }
  }

  /// 添加额外的配置项
  pub fn with_extra_config(mut self, key: impl Into<String>, value: JsonValue) -> Self {
    if let JsonValue::Object(ref mut map) = self.config {
      map.insert(key.into(), value);
    }
    self
  }
}

impl Default for BinaryStorageConfig {
  fn default() -> Self {
    Self::memory()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_fs_config() {
    let config = BinaryStorageConfig::fs("/tmp");
    assert!(matches!(config.storage_type, StorageType::Fs));
    assert_eq!(config.root, "/tmp");
  }

  #[test]
  fn test_s3_config() {
    let config = BinaryStorageConfig::s3("test-bucket", "us-west-2", "key", "secret");
    assert!(matches!(config.storage_type, StorageType::S3));

    if let JsonValue::Object(map) = &config.config {
      assert_eq!(map.get("bucket").unwrap(), "test-bucket");
      assert_eq!(map.get("region").unwrap(), "us-west-2");
    } else {
      panic!("Expected object config");
    }
  }

  #[test]
  fn test_memory_config() {
    let config = BinaryStorageConfig::memory();
    assert!(matches!(config.storage_type, StorageType::Memory));
  }

  #[test]
  fn test_redis_config() {
    let config = BinaryStorageConfig::redis("localhost:6379", 0);
    assert!(matches!(config.storage_type, StorageType::Redis));

    if let JsonValue::Object(map) = &config.config {
      assert_eq!(map.get("endpoint").unwrap(), "localhost:6379");
      assert_eq!(map.get("db").unwrap(), 0);
    } else {
      panic!("Expected object config");
    }
  }

  #[test]
  fn test_with_extra_config() {
    let config =
      BinaryStorageConfig::fs("/tmp").with_extra_config("custom_key", JsonValue::String("custom_value".to_string()));

    if let JsonValue::Object(map) = &config.config {
      assert_eq!(map.get("custom_key").unwrap(), "custom_value");
    } else {
      panic!("Expected object config");
    }
  }
}
