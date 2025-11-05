//! 二进制数据存储抽象层

use crate::binary_storage::{BinaryDataMetadata, BinaryStorageError};
use async_trait::async_trait;

/// 二进制数据存储抽象
///
/// 这个 trait 定义了二进制数据存储的统一接口，支持多种存储后端。
/// 实现者需要确保所有操作都是线程安全的。
#[async_trait]
pub trait BinaryDataStorage: Send + Sync {
  /// 存储二进制数据并返回引用
  ///
  /// # 参数
  /// - `data`: 要存储的二进制数据
  /// - `metadata`: 数据元信息
  ///
  /// # 返回
  /// - `Ok(String)`: 数据的唯一标识符
  /// - `Err(BinaryStorageError)`: 存储失败时的错误信息
  async fn store(&self, data: Vec<u8>, metadata: &BinaryDataMetadata) -> Result<String, BinaryStorageError>;

  /// 根据引用获取二进制数据
  ///
  /// # 参数
  /// - `key`: 数据的唯一标识符
  ///
  /// # 返回
  /// - `Ok(Vec<u8>)`: 二进制数据
  /// - `Err(BinaryStorageError)`: 获取失败时的错误信息
  async fn retrieve(&self, key: &str) -> Result<Vec<u8>, BinaryStorageError>;

  /// 获取文件元数据
  ///
  /// # 参数
  /// - `key`: 数据的唯一标识符
  ///
  /// # 返回
  /// - `Ok(BinaryDataMetadata)`: 文件元数据
  /// - `Err(BinaryStorageError)`: 获取失败时的错误信息
  async fn get_metadata(&self, key: &str) -> Result<BinaryDataMetadata, BinaryStorageError>;

  /// 删除二进制数据
  ///
  /// # 参数
  /// - `key`: 数据的唯一标识符
  ///
  /// # 返回
  /// - `Ok(())`: 删除成功
  /// - `Err(BinaryStorageError)`: 删除失败时的错误信息
  async fn delete(&self, key: &str) -> Result<(), BinaryStorageError>;

  /// 检查二进制数据是否存在
  ///
  /// # 参数
  /// - `key`: 数据的唯一标识符
  ///
  /// # 返回
  /// - `Ok(bool)`: 数据是否存在
  /// - `Err(BinaryStorageError)`: 检查失败时的错误信息
  async fn exists(&self, key: &str) -> Result<bool, BinaryStorageError>;

  /// 列出目录下的文件
  ///
  /// # 参数
  /// - `prefix`: 目录前缀
  ///
  /// # 返回
  /// - `Ok(Vec<String>)`: 文件键列表
  /// - `Err(BinaryStorageError)`: 列出失败时的错误信息
  async fn list(&self, prefix: &str) -> Result<Vec<String>, BinaryStorageError>;

  /// 获取存储类型名称
  ///
  /// # 返回
  /// - `&'static str`: 存储类型的名称
  fn storage_type_name(&self) -> &'static str;

  /// 检查存储是否可用
  ///
  /// # 返回
  /// - `Ok(bool)`: 存储是否可用
  /// - `Err(BinaryStorageError)`: 检查失败时的错误信息
  async fn is_available(&self) -> Result<bool, BinaryStorageError> {
    // 默认实现：尝试获取一个不存在的键，如果返回特定错误则认为存储可用
    match self.exists("__storage_availability_test__").await {
      Ok(_) => Ok(true),
      Err(BinaryStorageError::FileNotFound(_)) => Ok(true),
      Err(_) => Ok(false),
    }
  }
}

/// 存储能力标志
#[derive(Debug, Clone)]
pub struct StorageCapabilities {
  /// 是否支持并发写入
  pub supports_concurrent_writes: bool,
  /// 是否支持部分读取（范围读取）
  pub supports_range_read: bool,
  /// 是否支持流式写入
  pub supports_streaming_write: bool,
  /// 是否支持原子操作
  pub supports_atomic_operations: bool,
  /// 最大文件大小限制（字节），None表示无限制
  pub max_file_size: Option<usize>,
  /// 支持的文件类型MIME列表，空表示支持所有类型
  pub supported_mime_types: Vec<String>,
}

impl Default for StorageCapabilities {
  fn default() -> Self {
    Self {
      supports_concurrent_writes: true,
      supports_range_read: false,
      supports_streaming_write: false,
      supports_atomic_operations: false,
      max_file_size: None,
      supported_mime_types: Vec::new(),
    }
  }
}

impl StorageCapabilities {
  /// 创建本地文件系统存储的能力标志
  pub fn for_filesystem() -> Self {
    Self {
      supports_concurrent_writes: false,
      supports_range_read: true,
      supports_streaming_write: true,
      supports_atomic_operations: false,
      max_file_size: None,
      supported_mime_types: Vec::new(),
    }
  }

  /// 创建对象存储（如S3）的能力标志
  pub fn for_object_storage() -> Self {
    Self {
      supports_concurrent_writes: true,
      supports_range_read: true,
      supports_streaming_write: true,
      supports_atomic_operations: true,
      max_file_size: Some(5 * 1024 * 1024 * 1024), // 5GB
      supported_mime_types: Vec::new(),
    }
  }

  /// 创建内存存储的能力标志
  pub fn for_memory() -> Self {
    Self {
      supports_concurrent_writes: true,
      supports_range_read: false,
      supports_streaming_write: false,
      supports_atomic_operations: true,
      max_file_size: Some(100 * 1024 * 1024), // 100MB
      supported_mime_types: Vec::new(),
    }
  }

  /// 检查是否支持指定的MIME类型
  pub fn supports_mime_type(&self, mime_type: &str) -> bool {
    self.supported_mime_types.is_empty()
      || self
        .supported_mime_types
        .iter()
        .any(|supported| mime_type == supported || mime_type.starts_with(&format!("{}/", supported)))
  }

  /// 检查文件大小是否在限制范围内
  pub fn is_file_size_allowed(&self, size: usize) -> bool {
    match self.max_file_size {
      Some(max_size) => size <= max_size,
      None => true,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // 创建一个测试用的存储实现
  struct TestStorage {
    name: &'static str,
  }

  #[async_trait]
  impl BinaryDataStorage for TestStorage {
    async fn store(&self, _data: Vec<u8>, _metadata: &BinaryDataMetadata) -> Result<String, BinaryStorageError> {
      Ok("test_key".to_string())
    }

    async fn retrieve(&self, _key: &str) -> Result<Vec<u8>, BinaryStorageError> {
      Ok(vec![1, 2, 3])
    }

    async fn get_metadata(&self, _key: &str) -> Result<BinaryDataMetadata, BinaryStorageError> {
      Ok(BinaryDataMetadata::default())
    }

    async fn delete(&self, _key: &str) -> Result<(), BinaryStorageError> {
      Ok(())
    }

    async fn exists(&self, _key: &str) -> Result<bool, BinaryStorageError> {
      Ok(true)
    }

    async fn list(&self, _prefix: &str) -> Result<Vec<String>, BinaryStorageError> {
      Ok(vec!["test_key".to_string()])
    }

    fn storage_type_name(&self) -> &'static str {
      self.name
    }
  }

  #[tokio::test]
  async fn test_storage_interface() {
    let storage = TestStorage { name: "test" };

    // 测试存储
    let data = vec![1, 2, 3, 4];
    let metadata = BinaryDataMetadata::default();
    let key = storage.store(data, &metadata).await.unwrap();
    assert_eq!(key, "test_key");

    // 测试获取
    let retrieved = storage.retrieve(&key).await.unwrap();
    assert_eq!(retrieved, vec![1, 2, 3]);

    // 测试存在性检查
    let exists = storage.exists(&key).await.unwrap();
    assert!(exists);

    // 测试元数据获取
    let _meta = storage.get_metadata(&key).await.unwrap();

    // 测试列出
    let keys = storage.list("").await.unwrap();
    assert_eq!(keys, vec!["test_key"]);

    // 测试删除
    storage.delete(&key).await.unwrap();

    // 测试存储类型名称
    assert_eq!(storage.storage_type_name(), "test");

    // 测试可用性检查
    let available = storage.is_available().await.unwrap();
    assert!(available);
  }

  // 负向测试：模拟错误路径
  struct NegativeTestStorage;

  #[async_trait]
  impl BinaryDataStorage for NegativeTestStorage {
    async fn store(&self, _data: Vec<u8>, _metadata: &BinaryDataMetadata) -> Result<String, BinaryStorageError> {
      Err(BinaryStorageError::operation("store failed"))
    }

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, BinaryStorageError> {
      Err(BinaryStorageError::file_not_found(key))
    }

    async fn get_metadata(&self, _key: &str) -> Result<BinaryDataMetadata, BinaryStorageError> {
      Err(BinaryStorageError::file_not_found("missing"))
    }

    async fn delete(&self, _key: &str) -> Result<(), BinaryStorageError> {
      Err(BinaryStorageError::file_not_found("missing"))
    }

    async fn exists(&self, _key: &str) -> Result<bool, BinaryStorageError> {
      Err(BinaryStorageError::file_not_found("missing"))
    }

    async fn list(&self, _prefix: &str) -> Result<Vec<String>, BinaryStorageError> {
      Ok(vec![])
    }

    fn storage_type_name(&self) -> &'static str {
      "s3"
    }
  }

  #[tokio::test]
  async fn test_storage_negative_paths() {
    let storage = NegativeTestStorage;
    let err = storage.store(vec![1, 2, 3], &BinaryDataMetadata::default()).await.unwrap_err();
    match err {
      BinaryStorageError::OperationError(msg) => assert!(msg.contains("store failed")),
      _ => panic!("unexpected"),
    }

    let err = storage.retrieve("missing").await.unwrap_err();
    match err {
      BinaryStorageError::FileNotFound(path) => assert!(path.contains("missing")),
      _ => panic!("unexpected"),
    }

    // is_available 在 FileNotFound 时应返回可用
    let available = storage.is_available().await.unwrap();
    assert!(available);
  }

  #[test]
  #[allow(clippy::field_reassign_with_default)]
  fn test_storage_capabilities() {
    // 测试默认能力
    let default_caps = StorageCapabilities::default();
    assert!(default_caps.supports_concurrent_writes);
    assert!(!default_caps.supports_range_read);
    assert!(!default_caps.supports_streaming_write);
    assert!(!default_caps.supports_atomic_operations);
    assert!(default_caps.max_file_size.is_none());
    assert!(default_caps.supports_mime_type("application/json"));
    assert!(default_caps.is_file_size_allowed(1024));

    // 测试文件系统能力
    let fs_caps = StorageCapabilities::for_filesystem();
    assert!(!fs_caps.supports_concurrent_writes);
    assert!(fs_caps.supports_range_read);
    assert!(fs_caps.supports_streaming_write);
    assert!(!fs_caps.supports_atomic_operations);

    // 测试对象存储能力
    let s3_caps = StorageCapabilities::for_object_storage();
    assert!(s3_caps.supports_concurrent_writes);
    assert!(s3_caps.supports_range_read);
    assert!(s3_caps.supports_streaming_write);
    assert!(s3_caps.supports_atomic_operations);
    assert_eq!(s3_caps.max_file_size, Some(5 * 1024 * 1024 * 1024));

    // 测试内存存储能力
    let mem_caps = StorageCapabilities::for_memory();
    assert!(mem_caps.supports_concurrent_writes);
    assert!(!mem_caps.supports_range_read);
    assert!(!mem_caps.supports_streaming_write);
    assert!(mem_caps.supports_atomic_operations);
    assert_eq!(mem_caps.max_file_size, Some(100 * 1024 * 1024));

    // 测试MIME类型支持
    let mut caps = StorageCapabilities::default();
    caps.supported_mime_types.push("image".to_string());
    assert!(caps.supports_mime_type("image/jpeg"));
    assert!(caps.supports_mime_type("image/png"));
    assert!(!caps.supports_mime_type("text/plain"));

    // 测试文件大小限制
    let mut caps = StorageCapabilities::default();
    caps.max_file_size = Some(1000);
    assert!(caps.is_file_size_allowed(999));
    assert!(!caps.is_file_size_allowed(1001));
  }
}
