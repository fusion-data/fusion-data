//! 二进制数据存储配置

use serde::{Deserialize, Serialize};

/// 二进制数据存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStorageConfig {
  /// 存储类型
  pub kind: StorageKind,
  /// 根路径/桶名
  pub root: String,
  /// 缓存限制（字节）
  pub cache_limit: Option<usize>,

  pub bucket: Option<String>,
  pub region: Option<String>,
  pub access_key: Option<String>,
  pub secret_key: Option<String>,
  pub endpoint: Option<String>,
}

/// 支持的存储类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageKind {
  /// 本地文件系统
  Fs,
  /// S3兼容存储
  S3,
  /// 内存存储（主要用于测试）
  Memory,
}

impl BinaryStorageConfig {
  /// 创建本地文件系统配置
  pub fn fs(root: impl Into<String>) -> Self {
    Self {
      kind: StorageKind::Fs,
      root: root.into(),
      cache_limit: None,
      bucket: None,
      region: None,
      access_key: None,
      secret_key: None,
      endpoint: None,
    }
  }

  /// 创建内存存储配置
  pub fn memory(cache_limit: Option<usize>) -> Self {
    Self {
      kind: StorageKind::Memory,
      root: "memory".to_string(),
      cache_limit,
      bucket: None,
      region: None,
      access_key: None,
      secret_key: None,
      endpoint: None,
    }
  }

  /// 创建 S3 兼容对象存储配置
  ///
  /// 该构造函数用于初始化基于 S3/兼容对象存储的配置。为避免递归，请显式设置所有字段，
  /// 不要通过 `..Default::default()` 进行补全。
  ///
  /// 参数说明：
  /// - `bucket`: 桶名（同时会设置为 `root`）
  /// - `region`: 区域（可选）
  /// - `endpoint`: 端点地址（可选，适配 MinIO/Ceph 等兼容方案）
  /// - `access_key`: 访问密钥（可选）
  /// - `secret_key`: 密钥（可选）
  pub fn s3(
    bucket: impl Into<String>,
    region: Option<String>,
    endpoint: Option<String>,
    access_key: Option<String>,
    secret_key: Option<String>,
  ) -> Self {
    let bucket_str = bucket.into();
    Self {
      kind: StorageKind::S3,
      root: bucket_str.clone(),
      cache_limit: None,
      bucket: Some(bucket_str),
      region,
      access_key,
      secret_key,
      endpoint,
    }
  }
}

impl Default for BinaryStorageConfig {
  fn default() -> Self {
    Self {
      kind: StorageKind::Memory,
      root: "memory".to_string(),
      cache_limit: None,
      bucket: None,
      region: None,
      access_key: None,
      secret_key: None,
      endpoint: None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_fs_config() {
    let config = BinaryStorageConfig::fs("/tmp");
    assert!(matches!(config.kind, StorageKind::Fs));
    assert_eq!(config.root, "/tmp");
  }

  #[test]
  fn test_memory_config() {
    let config = BinaryStorageConfig::memory(None);
    assert!(matches!(config.kind, StorageKind::Memory));
  }

  #[test]
  fn test_s3_config() {
    let config = BinaryStorageConfig::s3(
      "my-bucket",
      Some("us-east-1".to_string()),
      Some("https://s3.example.com".to_string()),
      Some("AKIA...".to_string()),
      Some("SECRET".to_string()),
    );
    assert!(matches!(config.kind, StorageKind::S3));
    assert_eq!(config.root, "my-bucket");
    assert_eq!(config.bucket, Some("my-bucket".to_string()));
    assert_eq!(config.region, Some("us-east-1".to_string()));
    assert_eq!(config.endpoint, Some("https://s3.example.com".to_string()));
    assert_eq!(config.access_key, Some("AKIA...".to_string()));
    assert_eq!(config.secret_key, Some("SECRET".to_string()));
  }
}
