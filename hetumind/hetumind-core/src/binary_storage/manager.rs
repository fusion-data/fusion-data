//! 二进制数据管理器

use std::sync::Arc;

use ahash::HashMap;
use log::debug;
use tokio::sync::RwLock;

use crate::binary_storage::{
  BasicMetricsCollector, BinaryDataMetadata, BinaryDataStorage, BinaryStorageError, StorageKind,
};
use crate::types::BinaryFileKind;
use crate::workflow::BinaryDataReference;

/// 二进制数据管理器
///
/// 负责管理二进制数据的存储、检索和缓存。
#[derive(Clone)]
pub struct BinaryDataManager {
  /// 存储实现
  storage: Arc<dyn BinaryDataStorage>,
  /// 缓存 (可选)
  cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
  /// 缓存大小限制
  cache_limit: usize,
  /// 当前缓存大小
  cache_size: Arc<RwLock<usize>>,
  /// 存储类型
  storage_type: StorageKind,
  /// 指标收集器
  metrics_collector: Arc<BasicMetricsCollector>,
}

impl BinaryDataManager {
  /// 创建新的二进制数据管理器
  pub fn new(storage: Arc<dyn BinaryDataStorage>, cache_limit: usize) -> Result<Self, BinaryStorageError> {
    let storage_type = match storage.storage_type_name() {
      "fs" => StorageKind::Fs,
      "s3" => StorageKind::S3,
      _ => StorageKind::Memory,
    };

    Ok(Self {
      storage,
      cache: Arc::new(RwLock::new(HashMap::default())),
      cache_limit,
      cache_size: Arc::new(RwLock::new(0)),
      storage_type,
      metrics_collector: Arc::new(BasicMetricsCollector::new()),
    })
  }

  /// 创建带默认缓存限制的管理器
  pub fn with_default_cache(storage: Arc<dyn BinaryDataStorage>) -> Result<Self, BinaryStorageError> {
    Self::new(storage, 100 * 1024 * 1024) // 100MB默认缓存
  }

  /// 存储二进制数据并创建引用
  pub async fn store_data(
    &self,
    data: Vec<u8>,
    metadata: BinaryDataMetadata,
  ) -> Result<BinaryDataReference, BinaryStorageError> {
    let operation_id = self.metrics_collector.start_operation_async("store", data.len()).await;

    // 检查存储容量
    if !self.storage.is_available().await? {
      let metrics_collector = self.metrics_collector.clone();
      let operation_id_clone = operation_id.clone();
      tokio::spawn(async move {
        metrics_collector.set_operation_error(&operation_id_clone, "存储不可用".to_string()).await.ok();
      });
      return Err(BinaryStorageError::operation("存储不可用"));
    }

    // 存储数据
    let key = self.storage.store(data.clone(), &metadata).await.inspect_err(|e| {
      let metrics_collector = self.metrics_collector.clone();
      let operation_id_clone = operation_id.clone();
      let error = e.to_string();
      tokio::spawn(async move {
        metrics_collector.set_operation_error(&operation_id_clone, error).await.ok();
      });
    })?;

    let bytes_processed = data.len() as u64;

    // 更新缓存（如果需要）
    if self.should_cache(&data) {
      self.update_cache(&key, data).await?;
    }

    // 创建二进制数据引用
    let reference = BinaryDataReference {
      file_key: key.clone(),
      mime_kind: metadata.mime_type.clone(),
      file_size: metadata.file_size,
      file_name: metadata.file_name.clone(),
      file_kind: metadata.file_kind.or_else(|| Some(self.determine_file_kind(&metadata.mime_type))),
      file_extension: metadata.file_extension.clone(),
      directory: metadata.directory.clone(),
    };

    self.metrics_collector.complete_operation(&operation_id, true, bytes_processed).await?;
    debug!("存储二进制数据成功: key={}, size={}", key, bytes_processed);

    Ok(reference)
  }

  /// 获取二进制数据
  pub async fn get_data(&self, key: &str) -> Result<Vec<u8>, BinaryStorageError> {
    let operation_id = self.metrics_collector.start_operation_async("retrieve", 0).await;

    // 先检查缓存
    {
      let cache = self.cache.read().await;
      if let Some(data) = cache.get(key) {
        self.metrics_collector.complete_operation(&operation_id, true, data.len() as u64).await?;
        debug!("从缓存获取二进制数据: key={}, size={}", key, data.len());
        return Ok(data.clone());
      }
    }

    // 从存储获取
    let data = self.storage.retrieve(key).await.inspect_err(|e| {
      let metrics_collector = self.metrics_collector.clone();
      let operation_id_clone = operation_id.clone();
      let error = e.to_string();
      tokio::spawn(async move {
        metrics_collector.set_operation_error(&operation_id_clone, error).await.ok();
      });
    })?;

    let byte_processed = data.len() as u64;

    // 更新缓存
    if self.should_cache(&data) {
      self.update_cache(key, data.clone()).await?;
    }

    self.metrics_collector.complete_operation(&operation_id, true, byte_processed).await?;
    debug!("从存储获取二进制数据: key={}, size={}", key, byte_processed);

    Ok(data)
  }

  /// 删除二进制数据
  pub async fn delete_data(&self, key: &str) -> Result<(), BinaryStorageError> {
    let operation_id = self.metrics_collector.start_operation_async("delete", 0).await;

    // 从缓存删除
    {
      let mut cache = self.cache.write().await;
      if let Some(data) = cache.remove(key) {
        let mut cache_size = self.cache_size.write().await;
        *cache_size = cache_size.saturating_sub(data.len());
        debug!("从缓存删除二进制数据: key={}, size={}", key, data.len());
      }
    }

    // 从存储删除
    let result = self.storage.delete(key).await.inspect_err(|e| {
      let metrics_collector = self.metrics_collector.clone();
      let operation_id_clone = operation_id.clone();
      let error = e.to_string();
      tokio::spawn(async move {
        metrics_collector.set_operation_error(&operation_id_clone, error).await.ok();
      });
    });

    match &result {
      Ok(()) => {
        self.metrics_collector.complete_operation(&operation_id, true, 0).await?;
        debug!("删除二进制数据成功: key={}", key);
      }
      Err(e) => {
        debug!("删除二进制数据失败: key={}, error={}", key, e);
      }
    }

    result
  }

  /// 检查数据是否存在
  pub async fn data_exists(&self, key: &str) -> Result<bool, BinaryStorageError> {
    // 先检查缓存
    {
      let cache = self.cache.read().await;
      if cache.contains_key(key) {
        return Ok(true);
      }
    }

    // 检查存储
    self.storage.exists(key).await
  }

  /// 获取文件元数据
  pub async fn get_metadata(&self, key: &str) -> Result<BinaryDataMetadata, BinaryStorageError> {
    self.storage.get_metadata(key).await
  }

  /// 列出目录下的文件
  pub async fn list_files(&self, prefix: &str) -> Result<Vec<String>, BinaryStorageError> {
    self.storage.list(prefix).await
  }

  /// 获取操作进度
  pub async fn get_operation_progress(&self, operation_id: &str) -> Option<crate::binary_storage::OperationProgress> {
    self.metrics_collector.get_progress(operation_id).await
  }

  /// 获取所有当前操作
  pub async fn get_all_operations(&self) -> Vec<crate::binary_storage::OperationProgress> {
    self.metrics_collector.get_all_operations().await
  }

  /// 获取统计信息
  pub async fn get_stats(&self) -> crate::binary_storage::BasicStats {
    self.metrics_collector.get_stats().await
  }

  /// 清理缓存
  pub async fn cleanup_cache(&self, target_size: Option<usize>) {
    let mut cache = self.cache.write().await;
    let mut cache_size = self.cache_size.write().await;

    match target_size {
      Some(target) => {
        if *cache_size <= target {
          return;
        }

        // 计算需要删除的数据量
        let to_remove = *cache_size - target;
        let mut removed = 0;

        // 简单策略：删除一些条目直到达到目标大小
        let keys_to_remove: Vec<String> = cache
                    .keys()
                    .take(10) // 每次最多删除10个条目
                    .cloned()
                    .collect();

        for key in keys_to_remove {
          if let Some(data) = cache.remove(&key) {
            removed += data.len();
            if removed >= to_remove {
              break;
            }
          }
        }

        *cache_size = cache_size.saturating_sub(removed);
        debug!("缓存清理完成: 删除 {} 字节，当前大小 {} 字节", removed, *cache_size);
      }
      None => {
        // 完全清空缓存
        let removed = *cache_size;
        cache.clear();
        *cache_size = 0;
        debug!("缓存完全清空: 删除 {} 字节", removed);
      }
    }
  }

  /// 获取缓存信息
  pub async fn get_cache_info(&self) -> (usize, usize) {
    let cache_size = *self.cache_size.read().await;
    (cache_size, self.cache_limit)
  }

  /// 更新缓存
  async fn update_cache(&self, key: &str, data: Vec<u8>) -> Result<(), BinaryStorageError> {
    let mut cache = self.cache.write().await;
    let mut cache_size = self.cache_size.write().await;

    // 如果缓存已满，先清理一些空间
    if *cache_size + data.len() > self.cache_limit {
      drop(cache);
      drop(cache_size);
      // 清理1/4的空间
      let target_size = self.cache_limit - self.cache_limit / 4;
      self.cleanup_cache(Some(target_size)).await;

      // 重新获取锁
      cache = self.cache.write().await;
      cache_size = self.cache_size.write().await;

      // 如果仍然无法容纳，跳过缓存
      if *cache_size + data.len() > self.cache_limit {
        debug!("缓存已满，跳过缓存: key={}, size={}", key, data.len());
        return Ok(());
      }
    }

    // 更新缓存
    if let Some(existing_data) = cache.get(key) {
      *cache_size = cache_size.saturating_sub(existing_data.len());
    }

    cache.insert(key.to_string(), data.clone());
    *cache_size += data.len();

    debug!("更新缓存: key={}, size={}", key, data.len());
    Ok(())
  }

  /// 确定是否应该缓存数据
  fn should_cache(&self, data: &[u8]) -> bool {
    // 只缓存小文件和非内存存储
    match self.storage_type {
      StorageKind::Memory => false,       // 内存存储不需要缓存
      _ => data.len() < 10 * 1024 * 1024, // 只缓存小于10MB的文件
    }
  }

  /// 根据MIME类型确定文件类型
  fn determine_file_kind(&self, mime_type: &str) -> BinaryFileKind {
    match mime_type {
      t if t.starts_with("text/") && mime_type != "text/html" => BinaryFileKind::Text,
      "application/json" => BinaryFileKind::Json,
      t if t.starts_with("image/") => BinaryFileKind::Image,
      t if t.starts_with("video/") => BinaryFileKind::Video,
      t if t.starts_with("audio/") => BinaryFileKind::Audio,
      "application/pdf" => BinaryFileKind::Pdf,
      "text/html" => BinaryFileKind::Html,
      t if t.contains("sheet") || t.contains("excel") => BinaryFileKind::Excel,
      t if t.contains("word") || t.contains("document") => BinaryFileKind::Word,
      t if t.contains("presentation") || t.contains("powerpoint") => BinaryFileKind::Ppt,
      _ => BinaryFileKind::Text,
    }
  }

  /// 获取存储类型
  pub fn storage_type(&self) -> StorageKind {
    self.storage_type.clone()
  }

  /// 获取指标收集器
  pub fn get_metrics_collector(&self) -> Arc<BasicMetricsCollector> {
    self.metrics_collector.clone()
  }
}

#[cfg(test)]
mod tests {
  use async_trait::async_trait;

  use super::*;

  // 创建一个模拟的存储实现用于测试
  struct MockStorage {
    name: &'static str,
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
  }

  impl MockStorage {
    fn new(name: &'static str) -> Self {
      Self { name, data: Arc::new(RwLock::new(HashMap::default())) }
    }
  }

  #[async_trait]
  impl BinaryDataStorage for MockStorage {
    async fn store(&self, data: Vec<u8>, _metadata: &BinaryDataMetadata) -> Result<String, BinaryStorageError> {
      let key = format!("{}_{}", self.name, uuid::Uuid::new_v4());
      let mut data_map = self.data.write().await;
      data_map.insert(key.clone(), data);
      Ok(key)
    }

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, BinaryStorageError> {
      let data_map = self.data.read().await;
      data_map.get(key).cloned().ok_or_else(|| BinaryStorageError::file_not_found(key))
    }

    async fn get_metadata(&self, key: &str) -> Result<BinaryDataMetadata, BinaryStorageError> {
      let data_map = self.data.read().await;
      if data_map.contains_key(key) {
        Ok(BinaryDataMetadata::default())
      } else {
        Err(BinaryStorageError::file_not_found(key))
      }
    }

    async fn delete(&self, key: &str) -> Result<(), BinaryStorageError> {
      let mut data_map = self.data.write().await;
      data_map.remove(key).map(|_| ()).ok_or_else(|| BinaryStorageError::file_not_found(key))
    }

    async fn exists(&self, key: &str) -> Result<bool, BinaryStorageError> {
      let data_map = self.data.read().await;
      Ok(data_map.contains_key(key))
    }

    async fn list(&self, _prefix: &str) -> Result<Vec<String>, BinaryStorageError> {
      let data_map = self.data.read().await;
      Ok(data_map.keys().cloned().collect())
    }

    fn storage_type_name(&self) -> &'static str {
      self.name
    }
  }

  #[tokio::test]
  async fn test_data_manager() {
    let storage = Arc::new(MockStorage::new("test"));
    let manager = BinaryDataManager::with_default_cache(storage).unwrap();

    // 测试存储数据
    let data = vec![1, 2, 3, 4, 5];
    let metadata = BinaryDataMetadata::new(Some("test.txt".to_string()), "text/plain".to_string(), data.len() as u64);

    let reference = manager.store_data(data.clone(), metadata).await.unwrap();
    assert_eq!(reference.file_size, data.len() as u64);
    assert_eq!(reference.mime_kind, "text/plain");

    // 测试获取数据
    let retrieved = manager.get_data(&reference.file_key).await.unwrap();
    assert_eq!(retrieved, data);

    // 测试数据存在性
    assert!(manager.data_exists(&reference.file_key).await.unwrap());

    // 测试删除数据
    manager.delete_data(&reference.file_key).await.unwrap();
    assert!(!manager.data_exists(&reference.file_key).await.unwrap());
  }

  #[tokio::test]
  async fn test_cache_behavior() {
    let storage = Arc::new(MockStorage::new("test"));
    let manager = BinaryDataManager::new(storage, 100).unwrap(); // 100字节缓存

    // 存储小数据（应该被缓存）
    let small_data = vec![1; 50];
    let metadata =
      BinaryDataMetadata::new(Some("small.txt".to_string()), "text/plain".to_string(), small_data.len() as u64);

    let reference1 = manager.store_data(small_data.clone(), metadata).await.unwrap();

    // 获取数据（应该从缓存获取）
    let retrieved = manager.get_data(&reference1.file_key).await.unwrap();
    assert_eq!(retrieved, small_data);

    // 存储大数据（不应该被缓存）
    let large_data = vec![1; 200];
    let metadata =
      BinaryDataMetadata::new(Some("large.txt".to_string()), "text/plain".to_string(), large_data.len() as u64);

    let reference2 = manager.store_data(large_data.clone(), metadata).await.unwrap();

    // 获取数据（应该从存储获取）
    let retrieved = manager.get_data(&reference2.file_key).await.unwrap();
    assert_eq!(retrieved, large_data);

    // 检查缓存信息
    let (cache_size, cache_limit) = manager.get_cache_info().await;
    assert_eq!(cache_limit, 100);
    assert!(cache_size <= 100);
  }

  #[tokio::test]
  async fn test_metrics() {
    let storage = Arc::new(MockStorage::new("test"));
    let manager = BinaryDataManager::with_default_cache(storage).unwrap();

    // 重置统计信息以确保从零开始
    manager.get_metrics_collector().reset_stats().await;

    // 执行一些操作
    let data = vec![1, 2, 3, 4, 5];
    let metadata = BinaryDataMetadata::default();

    let reference = manager.store_data(data, metadata).await.unwrap();
    let _ = manager.get_data(&reference.file_key).await.unwrap();
    manager.delete_data(&reference.file_key).await.unwrap();

    // 检查统计信息
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_operations, 3);
    assert_eq!(stats.successful_operations, 3);
    assert_eq!(stats.failed_operations, 0);
  }

  #[tokio::test]
  async fn test_determine_file_kind() {
    let storage = Arc::new(MockStorage::new("test"));
    let manager = BinaryDataManager::with_default_cache(storage).unwrap();

    // 测试各种MIME类型
    assert_eq!(manager.determine_file_kind("text/plain"), BinaryFileKind::Text);
    assert_eq!(manager.determine_file_kind("application/json"), BinaryFileKind::Json);
    assert_eq!(manager.determine_file_kind("image/jpeg"), BinaryFileKind::Image);
    assert_eq!(manager.determine_file_kind("video/mp4"), BinaryFileKind::Video);
    assert_eq!(manager.determine_file_kind("audio/mp3"), BinaryFileKind::Audio);
    assert_eq!(manager.determine_file_kind("application/pdf"), BinaryFileKind::Pdf);
    assert_eq!(manager.determine_file_kind("text/html"), BinaryFileKind::Html);
    assert_eq!(manager.determine_file_kind("application/vnd.ms-excel"), BinaryFileKind::Excel);
    assert_eq!(manager.determine_file_kind("application/msword"), BinaryFileKind::Word);
    assert_eq!(manager.determine_file_kind("application/vnd.ms-powerpoint"), BinaryFileKind::Ppt);
    assert_eq!(manager.determine_file_kind("unknown/type"), BinaryFileKind::Text);
  }
}
