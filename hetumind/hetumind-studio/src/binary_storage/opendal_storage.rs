//! 基于 opendal 的存储实现

use async_trait::async_trait;
use hetumind_core::binary_storage::{
  BinaryDataMetadata, BinaryDataStorage, BinaryStorageError, StorageCapabilities, StorageType,
};
use log::debug;
use opendal::{ErrorKind, Operator};
use uuid::Uuid;

/// 基于 opendal 的存储实现
pub struct OpenDalStorage {
  operator: Operator,
  storage_type: StorageType,
  capabilities: StorageCapabilities,
}

impl OpenDalStorage {
  /// 创建新的存储实例
  ///
  /// # 参数
  /// - `operator`: opendal 操作符
  /// - `storage_type`: 存储类型
  ///
  /// # 返回
  /// - `Result<Self, BinaryStorageError>`: 创建结果
  pub fn new(operator: Operator, storage_type: StorageType) -> Result<Self, BinaryStorageError> {
    let capabilities = match storage_type {
      StorageType::Fs => StorageCapabilities::for_filesystem(),
      StorageType::S3 => StorageCapabilities::for_object_storage(),
      StorageType::Memory => StorageCapabilities::for_memory(),
      StorageType::Redis => StorageCapabilities::for_memory(), // Redis 类似于内存存储
    };

    Ok(Self { operator, storage_type, capabilities })
  }

  /// 生成唯一文件键
  ///
  /// # 参数
  /// - `metadata`: 数据元信息
  ///
  /// # 返回
  /// - `String`: 唯一文件键
  fn generate_file_key(&self, _metadata: &BinaryDataMetadata) -> String {
    // TODO 需要将 metadata 里的某些属性也作为 file key 的生成条件吗？
    Uuid::now_v7().to_string()
  }

  /// 验证数据是否可以存储
  ///
  /// # 参数
  /// - `data`: 数据内容
  /// - `metadata`: 数据元信息
  ///
  /// # 返回
  /// - `Result<(), BinaryStorageError>`: 验证结果
  fn validate_data(&self, data: &[u8], metadata: &BinaryDataMetadata) -> Result<(), BinaryStorageError> {
    // 检查文件大小
    if !self.capabilities.is_file_size_allowed(data.len()) {
      return Err(BinaryStorageError::operation(format!(
        "文件大小超出限制: {} > {:?}",
        data.len(),
        self.capabilities.max_file_size
      )));
    }

    // 检查MIME类型
    if !self.capabilities.supports_mime_type(&metadata.mime_type) {
      return Err(BinaryStorageError::operation(format!("不支持的MIME类型: {}", metadata.mime_type)));
    }

    Ok(())
  }
}

#[async_trait]
impl BinaryDataStorage for OpenDalStorage {
  async fn store(&self, data: Vec<u8>, metadata: &BinaryDataMetadata) -> Result<String, BinaryStorageError> {
    // 验证数据
    self.validate_data(&data, metadata)?;

    let key = self.generate_file_key(metadata);
    debug!("存储二进制数据: key={}, size={}", key, data.len());

    // 使用 opendal 写入数据
    self.operator.write(&key, data).await.map_err(|e| BinaryStorageError::operation(e.to_string()))?;

    Ok(key)
  }

  async fn retrieve(&self, key: &str) -> Result<Vec<u8>, BinaryStorageError> {
    debug!("检索二进制数据: key={}", key);

    let buffer = self.operator.read(key).await.map_err(|e| BinaryStorageError::operation(e.to_string()))?;
    let data = buffer.to_vec();
    Ok(data)
  }

  async fn get_metadata(&self, key: &str) -> Result<BinaryDataMetadata, BinaryStorageError> {
    debug!("获取二进制数据元数据: key={}", key);

    let meta = self.operator.stat(key).await.map_err(|e| BinaryStorageError::operation(e.to_string()))?;

    let file_name = Some(key.split('/').next_back().unwrap_or(key).to_string());
    let mime_type = meta.content_type().unwrap_or("application/octet-stream").to_string();
    let file_size = meta.content_length() as u64;
    let last_modified = meta.last_modified().map(|dt| dt.timestamp());

    // 推断文件类型和扩展名
    let mut metadata =
      BinaryDataMetadata::new(file_name, mime_type, file_size).with_last_modified(last_modified.unwrap_or(0));

    metadata.infer_type_and_extension();

    Ok(metadata)
  }

  async fn delete(&self, key: &str) -> Result<(), BinaryStorageError> {
    debug!("删除二进制数据: key={}", key);

    self.operator.delete(key).await.map_err(|e| BinaryStorageError::operation(e.to_string()))
  }

  async fn exists(&self, key: &str) -> Result<bool, BinaryStorageError> {
    debug!("检查二进制数据是否存在: key={}", key);

    match self.operator.stat(key).await {
      Ok(_) => Ok(true),
      Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
      Err(e) => Err(BinaryStorageError::operation(e.to_string())),
    }
  }

  async fn list(&self, prefix: &str) -> Result<Vec<String>, BinaryStorageError> {
    debug!("列出二进制数据: prefix={}", prefix);

    let lister = self.operator.list(prefix).await.map_err(|e| BinaryStorageError::operation(e.to_string()))?;
    let keys = lister.into_iter().map(|entry| entry.path().to_string()).collect();
    Ok(keys)
  }

  fn storage_type_name(&self) -> &'static str {
    match self.storage_type {
      StorageType::Fs => "fs",
      StorageType::S3 => "s3",
      StorageType::Memory => "memory",
      StorageType::Redis => "redis",
    }
  }
}

/// 创建文件系统存储
///
/// # 参数
/// - `root`: 根目录路径
///
/// # 返回
/// - `Result<OpenDalStorage, BinaryStorageError>`: 存储实例
pub async fn create_fs_storage(root: &str) -> Result<OpenDalStorage, BinaryStorageError> {
  let builder = opendal::services::Fs::default().root(root);
  let operator = Operator::new(builder).map_err(|e| BinaryStorageError::config(e.to_string()))?.finish();
  OpenDalStorage::new(operator, StorageType::Fs)
}

/// 创建S3存储
///
/// # 参数
/// - `bucket`: S3桶名
/// - `region`: AWS区域
/// - `access_key`: 访问密钥
/// - `secret_key`: 秘密密钥
/// - `endpoint`: 自定义端点（可选）
///
/// # 返回
/// - `Result<OpenDalStorage, BinaryStorageError>`: 存储实例
pub async fn create_s3_storage(
  bucket: &str,
  region: &str,
  access_key: &str,
  secret_key: &str,
  endpoint: Option<&str>,
) -> Result<OpenDalStorage, BinaryStorageError> {
  let mut builder = opendal::services::S3::default()
    .bucket(bucket)
    .region(region)
    .access_key_id(access_key)
    .secret_access_key(secret_key);
  if let Some(ep) = endpoint {
    builder = builder.endpoint(ep);
  }
  let operator = Operator::new(builder).map_err(|e| BinaryStorageError::config(e.to_string()))?.finish();
  OpenDalStorage::new(operator, StorageType::S3)
}

/// 创建内存存储
///
/// # 返回
/// - `Result<OpenDalStorage, BinaryStorageError>`: 存储实例
pub async fn create_memory_storage() -> Result<OpenDalStorage, BinaryStorageError> {
  let builder = opendal::services::Memory::default();
  let operator = Operator::new(builder).map_err(|e| BinaryStorageError::config(e.to_string()))?.finish();
  OpenDalStorage::new(operator, StorageType::Memory)
}

/// 创建Redis存储
///
/// # 参数
/// - `endpoint`: Redis端点
/// - `db`: 数据库编号
///
/// # 返回
/// - `Result<OpenDalStorage, BinaryStorageError>`: 存储实例
pub async fn create_redis_storage(endpoint: &str, db: i64) -> Result<OpenDalStorage, BinaryStorageError> {
  let builder = opendal::services::Redis::default().endpoint(endpoint).db(db);
  let operator = Operator::new(builder).map_err(|e| BinaryStorageError::config(e.to_string()))?.finish();
  OpenDalStorage::new(operator, StorageType::Redis)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_memory_storage() {
    let storage = create_memory_storage().await.unwrap();

    // 测试存储
    let data = b"Hello, World!".to_vec();
    let metadata = BinaryDataMetadata::new(Some("test.txt".to_string()), "text/plain".to_string(), data.len() as u64);

    let key = storage.store(data.clone(), &metadata).await.unwrap();

    // 测试检索
    let retrieved = storage.retrieve(&key).await.unwrap();
    assert_eq!(retrieved, data);

    // 测试元数据
    let retrieved_meta = storage.get_metadata(&key).await.unwrap();
    assert_eq!(retrieved_meta.file_size, data.len() as u64);

    // 测试存在性
    assert!(storage.exists(&key).await.unwrap());

    // 测试列出
    let keys = storage.list("").await.unwrap();
    assert!(keys.contains(&key));

    // 测试删除
    storage.delete(&key).await.unwrap();
    assert!(!storage.exists(&key).await.unwrap());
  }

  #[tokio::test]
  async fn test_file_key_generation() {
    let storage = create_memory_storage().await.unwrap();

    let metadata = BinaryDataMetadata::new(Some("test.txt".to_string()), "text/plain".to_string(), 100);

    let key1 = storage.generate_file_key(&metadata);
    let key2 = storage.generate_file_key(&metadata);

    // 确保生成的键是唯一的
    assert_ne!(key1, key2);
  }

  #[tokio::test]
  async fn test_storage_capabilities() {
    let fs_storage = create_memory_storage().await.unwrap();

    // 测试存储类型名称
    assert_eq!(fs_storage.storage_type_name(), "memory");

    // 测试能力验证
    let small_data = vec![1; 100];
    let large_data = vec![1; 200 * 1024 * 1024]; // 200MB

    let metadata = BinaryDataMetadata::default();

    // 小数据应该通过验证
    assert!(fs_storage.validate_data(&small_data, &metadata).is_ok());

    // 大数据应该不通过验证（内存存储限制100MB）
    assert!(fs_storage.validate_data(&large_data, &metadata).is_err());
  }
}
