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
    Self { kind: StorageKind::Fs, root: root.into(), cache_limit: None, ..Default::default() }
  }

  /// 创建内存存储配置
  pub fn memory(cache_limit: Option<usize>) -> Self {
    Self { kind: StorageKind::Memory, root: "memory".to_string(), cache_limit, ..Default::default() }
  }
}

impl Default for BinaryStorageConfig {
  fn default() -> Self {
    Self::memory(None)
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
}
