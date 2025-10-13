//! 二进制数据引用的生命周期管理
use std::sync::Arc;
use std::time::{Duration, Instant};

use ahash::HashMap;
use log::{debug, info, warn};
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::binary_storage::{BasicMetricsCollector, BinaryDataManager, BinaryDataMetadata, BinaryStorageError};

/// 二进制数据生命周期管理器
///
/// 负责跟踪二进制数据的引用情况，并在适当的时候清理无引用的数据。
pub struct BinaryDataLifecycleManager {
  /// 二进制数据管理器
  data_manager: BinaryDataManager,
  /// 引用计数
  reference_counts: Arc<RwLock<HashMap<String, usize>>>,
  /// 创建时间记录
  creation_times: Arc<RwLock<HashMap<String, Instant>>>,
  /// 最后访问时间记录
  last_access_times: Arc<RwLock<HashMap<String, Instant>>>,
  /// 清理配置
  cleanup_config: LifecycleCleanupConfig,
  /// 指标收集器
  metrics_collector: Arc<BasicMetricsCollector>,
  /// 是否已启动清理任务
  cleanup_task_started: Arc<RwLock<bool>>,
}

/// 生命周期清理配置
#[derive(Debug, Clone)]
pub struct LifecycleCleanupConfig {
  /// 引用计数为0后保留时间
  pub zero_ref_retention: Duration,
  /// 最大保留时间（无论引用计数）
  pub max_retention: Duration,
  /// 清理检查间隔
  pub cleanup_interval: Duration,
  /// 是否启用自动清理
  pub auto_cleanup: bool,
  /// 每次清理的最大数量（防止一次清理过多影响性能）
  pub max_cleanup_per_batch: usize,
}

impl Default for LifecycleCleanupConfig {
  fn default() -> Self {
    Self {
      zero_ref_retention: Duration::from_secs(3600), // 1小时
      max_retention: Duration::from_secs(86400 * 7), // 7天
      cleanup_interval: Duration::from_secs(300),    // 5分钟
      auto_cleanup: true,
      max_cleanup_per_batch: 100,
    }
  }
}

impl BinaryDataLifecycleManager {
  /// 创建新的生命周期管理器
  ///
  /// # 参数
  /// - `data_manager`: 二进制数据管理器
  /// - `cleanup_config`: 清理配置
  pub fn new(data_manager: BinaryDataManager, cleanup_config: LifecycleCleanupConfig) -> Self {
    Self {
      data_manager,
      reference_counts: Arc::new(RwLock::new(HashMap::default())),
      creation_times: Arc::new(RwLock::new(HashMap::default())),
      last_access_times: Arc::new(RwLock::new(HashMap::default())),
      cleanup_config,
      metrics_collector: Arc::new(BasicMetricsCollector::new()),
      cleanup_task_started: Arc::new(RwLock::new(false)),
    }
  }

  /// 创建带默认配置的生命周期管理器
  ///
  /// # 参数
  /// - `data_manager`: 二进制数据管理器
  pub fn with_default_config(data_manager: BinaryDataManager) -> Self {
    Self::new(data_manager, LifecycleCleanupConfig::default())
  }

  /// 注册新的二进制数据引用
  ///
  /// # 参数
  /// - `key`: 数据键
  ///
  /// # 返回
  /// - `Result<(), BinaryStorageError>`: 注册结果
  pub async fn register_reference(&self, key: &str) -> Result<(), BinaryStorageError> {
    let now = Instant::now();

    // 更新引用计数
    {
      let mut ref_counts = self.reference_counts.write().await;
      *ref_counts.entry(key.to_string()).or_insert(0) += 1;
    }

    // 记录创建时间（如果是新键）
    let is_new_key = {
      let mut creation_times = self.creation_times.write().await;
      if !creation_times.contains_key(key) {
        creation_times.insert(key.to_string(), now);
        debug!("注册新的二进制数据引用: {}", key);
        true
      } else {
        false
      }
    };

    // 更新最后访问时间
    {
      let mut access_times = self.last_access_times.write().await;
      access_times.insert(key.to_string(), now);
    }

    // 如果是新键，记录指标
    if is_new_key {
      self.metrics_collector.start_operation_async("register", 0).await;
      self.metrics_collector.complete_operation("register", true, 0).await.ok();
    }

    Ok(())
  }

  /// 释放二进制数据引用
  ///
  /// # 参数
  /// - `key`: 数据键
  ///
  /// # 返回
  /// - `Result<(), BinaryStorageError>`: 释放结果
  pub async fn release_reference(&self, key: &str) -> Result<(), BinaryStorageError> {
    let now = Instant::now();

    // 更新引用计数
    let should_remove = {
      let mut ref_counts = self.reference_counts.write().await;
      if let Some(count) = ref_counts.get_mut(key) {
        *count = count.saturating_sub(1);
        if *count == 0 {
          debug!("二进制数据引用计数归零: {}", key);
          true
        } else {
          false
        }
      } else {
        warn!("尝试释放不存在的引用: {}", key);
        false
      }
    };

    // 更新最后访问时间
    {
      let mut access_times = self.last_access_times.write().await;
      access_times.insert(key.to_string(), now);
    }

    // 如果引用计数为0且配置为立即清理，则执行清理
    if should_remove && self.cleanup_config.zero_ref_retention.is_zero() {
      self.cleanup_data(key).await?;
    }

    Ok(())
  }

  /// 获取二进制数据（自动更新访问时间）
  ///
  /// # 参数
  /// - `key`: 数据键
  ///
  /// # 返回
  /// - `Result<Vec<u8>, BinaryStorageError>`: 数据内容
  pub async fn get_data(&self, key: &str) -> Result<Vec<u8>, BinaryStorageError> {
    // 更新访问时间
    {
      let mut access_times = self.last_access_times.write().await;
      access_times.insert(key.to_string(), Instant::now());
    }

    // 获取数据
    self.data_manager.get_data(key).await
  }

  /// 存储二进制数据（自动注册引用）
  ///
  /// # 参数
  /// - `data`: 数据内容
  /// - `metadata`: 数据元信息
  ///
  /// # 返回
  /// - `Result<String, BinaryStorageError>`: 数据键
  pub async fn store_data(&self, data: Vec<u8>, metadata: BinaryDataMetadata) -> Result<String, BinaryStorageError> {
    // 存储数据
    let reference = self.data_manager.store_data(data, metadata).await?;

    // 注册引用
    self.register_reference(&reference.file_key).await?;

    Ok(reference.file_key)
  }

  /// 手动清理指定数据
  ///
  /// # 参数
  /// - `key`: 数据键
  ///
  /// # 返回
  /// - `Result<(), BinaryStorageError>`: 清理结果
  pub async fn cleanup_data(&self, key: &str) -> Result<(), BinaryStorageError> {
    info!("清理二进制数据: {}", key);

    let operation_id = self.metrics_collector.start_operation_async("cleanup", 0).await;

    // 检查引用计数
    let should_delete = {
      let ref_counts = self.reference_counts.read().await;
      ref_counts.get(key).is_none_or(|count| *count == 0)
    };

    if should_delete {
      // 删除数据
      let result = self.data_manager.delete_data(key).await;

      // 清理元数据
      {
        let mut ref_counts = self.reference_counts.write().await;
        ref_counts.remove(key);
      }
      {
        let mut creation_times = self.creation_times.write().await;
        creation_times.remove(key);
      }
      {
        let mut access_times = self.last_access_times.write().await;
        access_times.remove(key);
      }

      match result {
        Ok(()) => {
          info!("已清理二进制数据: {}", key);
          self.metrics_collector.complete_operation(&operation_id, true, 0).await?;
        }
        Err(e) => {
          self.metrics_collector.set_operation_error(&operation_id, e.to_string()).await?;
          return Err(e);
        }
      }
    } else {
      debug!("跳过清理，数据仍有引用: {}", key);
    }

    Ok(())
  }

  /// 启动自动清理任务
  ///
  /// # 返回
  /// - `Result<(), BinaryStorageError>`: 启动结果
  pub async fn start_cleanup_task(&self) -> Result<(), BinaryStorageError> {
    if !self.cleanup_config.auto_cleanup {
      return Ok(());
    }

    // 检查是否已经启动
    {
      let mut started = self.cleanup_task_started.write().await;
      if *started {
        info!("自动清理任务已经启动，跳过重复启动");
        return Ok(());
      }
      *started = true;
    }

    let cleanup_config = self.cleanup_config.clone();
    let reference_counts = self.reference_counts.clone();
    let creation_times = self.creation_times.clone();
    let last_access_times = self.last_access_times.clone();
    let data_manager = self.data_manager.clone();
    let metrics_collector = self.metrics_collector.clone();

    tokio::spawn(async move {
      let mut interval = interval(cleanup_config.cleanup_interval);

      loop {
        interval.tick().await;

        let operation_id = metrics_collector.start_operation_async("auto_cleanup", 0).await;

        let now = Instant::now();
        let mut keys_to_cleanup = Vec::new();

        // 收集需要清理的键
        {
          let ref_counts = reference_counts.read().await;
          let creation_times = creation_times.read().await;
          let last_access_times = last_access_times.read().await;

          for (key, &count) in ref_counts.iter() {
            let should_cleanup = if count == 0 {
              // 引用计数为0，检查零引用保留时间
              if let Some(last_access) = last_access_times.get(key) {
                now.duration_since(*last_access) > cleanup_config.zero_ref_retention
              } else {
                true
              }
            } else {
              // 检查最大保留时间
              if let Some(creation_time) = creation_times.get(key) {
                now.duration_since(*creation_time) > cleanup_config.max_retention
              } else {
                true
              }
            };

            if should_cleanup {
              keys_to_cleanup.push(key.clone());
            }
          }
        }

        // 限制每次清理的数量
        if keys_to_cleanup.len() > cleanup_config.max_cleanup_per_batch {
          keys_to_cleanup.truncate(cleanup_config.max_cleanup_per_batch);
        }

        let mut success_count = 0;
        let mut error_count = 0;

        // 执行清理
        for key in keys_to_cleanup {
          match data_manager.delete_data(&key).await {
            Ok(()) => {
              // 清理元数据
              {
                let mut ref_counts = reference_counts.write().await;
                ref_counts.remove(&key);
              }
              {
                let mut creation_times = creation_times.write().await;
                creation_times.remove(&key);
              }
              {
                let mut last_access_times = last_access_times.write().await;
                last_access_times.remove(&key);
              }

              info!("自动清理二进制数据: {}", key);
              success_count += 1;
            }
            Err(e) => {
              warn!("清理二进制数据失败 {}, error: {}", key, e);
              error_count += 1;
            }
          }
        }

        // 记录清理指标
        if success_count > 0 || error_count > 0 {
          metrics_collector.complete_operation(&operation_id, error_count == 0, 0).await.ok();
          info!("自动清理完成: 成功 {} 个, 失败 {} 个", success_count, error_count);
        } else {
          metrics_collector.complete_operation(&operation_id, true, 0).await.ok();
        }
      }
    });

    info!("已启动二进制数据自动清理任务");
    Ok(())
  }

  /// 获取引用计数
  ///
  /// # 参数
  /// - `key`: 数据键
  ///
  /// # 返回
  /// - `Option<usize>`: 引用计数，如果不存在则返回None
  pub async fn get_reference_count(&self, key: &str) -> Option<usize> {
    let ref_counts = self.reference_counts.read().await;
    ref_counts.get(key).copied()
  }

  /// 获取所有引用计数
  ///
  /// # 返回
  /// - `HashMap<String, usize>`: 所有键的引用计数
  pub async fn get_all_reference_counts(&self) -> HashMap<String, usize> {
    let ref_counts = self.reference_counts.read().await;
    ref_counts.clone()
  }

  /// 获取生命周期统计信息
  ///
  /// # 返回
  /// - `LifecycleStats`: 生命周期统计信息
  pub async fn get_lifecycle_stats(&self) -> LifecycleStats {
    let ref_counts = self.reference_counts.read().await;
    let creation_times = self.creation_times.read().await;
    let last_access_times = self.last_access_times.read().await;

    let total_refs: usize = ref_counts.values().sum();
    let zero_ref_count = ref_counts.values().filter(|&&count| count == 0).count();

    let now = Instant::now();
    let expired_zero_refs = ref_counts
      .iter()
      .filter(|(key, count)| {
        **count == 0
          && last_access_times
            .get(key.as_str())
            .is_none_or(|last_access| now.duration_since(*last_access) > self.cleanup_config.zero_ref_retention)
      })
      .count();

    LifecycleStats {
      total_files: ref_counts.len(),
      total_references: total_refs,
      zero_ref_files: zero_ref_count,
      expired_zero_ref_files: expired_zero_refs,
      oldest_file_age: creation_times.values().map(|&creation_time| now.duration_since(creation_time)).min(),
      newest_file_age: creation_times.values().map(|&creation_time| now.duration_since(creation_time)).max(),
    }
  }

  /// 获取指标收集器
  ///
  /// # 返回
  /// - `Arc<BasicMetricsCollector>`: 指标收集器的引用
  pub fn get_metrics_collector(&self) -> Arc<BasicMetricsCollector> {
    self.metrics_collector.clone()
  }
}

/// 生命周期统计信息
#[derive(Debug, Clone)]
pub struct LifecycleStats {
  /// 总文件数
  pub total_files: usize,
  /// 总引用数
  pub total_references: usize,
  /// 零引用文件数
  pub zero_ref_files: usize,
  /// 过期的零引用文件数
  pub expired_zero_ref_files: usize,
  /// 最旧文件年龄
  pub oldest_file_age: Option<Duration>,
  /// 最新文件年龄
  pub newest_file_age: Option<Duration>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_cleanup_config() {
    let config = LifecycleCleanupConfig::default();
    assert_eq!(config.zero_ref_retention, Duration::from_secs(3600));
    assert_eq!(config.max_retention, Duration::from_secs(86400 * 7));
    assert_eq!(config.cleanup_interval, Duration::from_secs(300));
    assert!(config.auto_cleanup);
    assert_eq!(config.max_cleanup_per_batch, 100);

    // 创建自定义配置
    let custom_config = LifecycleCleanupConfig {
      zero_ref_retention: Duration::from_secs(1800),
      max_retention: Duration::from_secs(86400 * 3),
      cleanup_interval: Duration::from_secs(600),
      auto_cleanup: false,
      max_cleanup_per_batch: 50,
    };

    assert_eq!(custom_config.zero_ref_retention, Duration::from_secs(1800));
    assert_eq!(custom_config.max_retention, Duration::from_secs(86400 * 3));
    assert_eq!(custom_config.cleanup_interval, Duration::from_secs(600));
    assert!(!custom_config.auto_cleanup);
    assert_eq!(custom_config.max_cleanup_per_batch, 50);
  }
}
