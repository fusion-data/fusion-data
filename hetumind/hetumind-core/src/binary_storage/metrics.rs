//! 基础指标收集

use crate::binary_storage::BinaryStorageError;
use fusion_common::ahash::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// 操作进度
#[derive(Debug, Clone)]
pub struct OperationProgress {
  /// 操作ID
  pub operation_id: String,
  /// 操作类型
  pub operation_type: String,
  /// 总字节数
  pub total_bytes: u64,
  /// 已处理字节数
  pub processed_bytes: u64,
  /// 开始时间
  pub start_time: Instant,
  /// 是否完成
  pub is_completed: bool,
  /// 错误信息
  pub error: Option<String>,
}

impl OperationProgress {
  /// 完成百分比
  pub fn completion_percentage(&self) -> f64 {
    if self.total_bytes == 0 { 0.0 } else { (self.processed_bytes as f64 / self.total_bytes as f64) * 100.0 }
  }

  /// 已用时间
  pub fn elapsed_time(&self) -> Duration {
    self.start_time.elapsed()
  }

  /// 预估剩余时间（简单估算）
  pub fn estimated_remaining_time(&self) -> Option<Duration> {
    if self.processed_bytes == 0 || self.is_completed {
      return None;
    }

    let elapsed = self.elapsed_time();
    let rate = self.processed_bytes as f64 / elapsed.as_secs_f64();
    let remaining_bytes = self.total_bytes.saturating_sub(self.processed_bytes) as f64;

    if rate > 0.0 { Some(Duration::from_secs_f64(remaining_bytes / rate)) } else { None }
  }

  /// 更新进度
  pub fn update_progress(&mut self, processed_bytes: u64) {
    self.processed_bytes = processed_bytes;
  }

  /// 标记为完成
  pub fn mark_completed(&mut self, success: bool) {
    self.is_completed = true;
    if !success {
      self.error = Some("操作失败".to_string());
    }
  }

  /// 设置错误信息
  pub fn set_error(&mut self, error: String) {
    self.error = Some(error);
    self.is_completed = true;
  }
}

/// 基础统计信息
#[derive(Debug, Default, Clone)]
pub struct BasicStats {
  /// 总操作数
  pub total_operations: u64,
  /// 成功操作数
  pub successful_operations: u64,
  /// 失败操作数
  pub failed_operations: u64,
  /// 总处理字节数
  pub total_bytes_processed: u64,
}

impl BasicStats {
  /// 成功率
  pub fn success_rate(&self) -> f64 {
    if self.total_operations == 0 { 0.0 } else { self.successful_operations as f64 / self.total_operations as f64 }
  }

  /// 吞吐量（字节/秒）
  pub fn throughput_bytes_per_sec(&self, avg_time_ms: u64) -> f64 {
    if avg_time_ms == 0 { 0.0 } else { (self.total_bytes_processed as f64) / (avg_time_ms as f64 / 1000.0) }
  }

  /// 更新成功操作
  pub fn increment_successful(&mut self, bytes_processed: u64) {
    self.total_operations += 1;
    self.successful_operations += 1;
    self.total_bytes_processed += bytes_processed;
  }

  /// 更新失败操作
  pub fn increment_failed(&mut self) {
    self.total_operations += 1;
    self.failed_operations += 1;
  }
}

/// 基础指标收集器
pub struct BasicMetricsCollector {
  /// 当前操作
  operations: Arc<RwLock<HashMap<String, OperationProgress>>>,
  /// 统计信息
  stats: Arc<RwLock<BasicStats>>,
}

impl BasicMetricsCollector {
  /// 创建新的指标收集器
  pub fn new() -> Self {
    Self { operations: Arc::new(RwLock::new(HashMap::default())), stats: Arc::new(RwLock::new(BasicStats::default())) }
  }

  /// 开始操作
  ///
  /// # 参数
  /// - `operation_type`: 操作类型（如 "store", "retrieve", "delete"）
  /// - `total_bytes`: 总字节数
  ///
  /// # 返回
  /// - `String`: 操作ID
  pub fn start_operation(&self, operation_type: &str, total_bytes: usize) -> String {
    let operation_id = Uuid::now_v7().to_string();
    let progress = OperationProgress {
      operation_id: operation_id.clone(),
      operation_type: operation_type.to_string(),
      total_bytes: total_bytes as u64,
      processed_bytes: 0,
      start_time: Instant::now(),
      is_completed: false,
      error: None,
    };

    // 添加到操作列表
    let mut operations = self.operations.blocking_write();
    operations.insert(operation_id.clone(), progress);

    // 不在这里更新统计，而是在完成操作时更新
    operation_id
  }

  /// 开始操作（异步版本）
  pub async fn start_operation_async(&self, operation_type: &str, total_bytes: usize) -> String {
    let operation_id = Uuid::now_v7().to_string();
    let progress = OperationProgress {
      operation_id: operation_id.clone(),
      operation_type: operation_type.to_string(),
      total_bytes: total_bytes as u64,
      processed_bytes: 0,
      start_time: Instant::now(),
      is_completed: false,
      error: None,
    };

    // 添加到操作列表
    let mut operations = self.operations.write().await;
    operations.insert(operation_id.clone(), progress);

    // 不在这里更新统计，而是在完成操作时更新
    operation_id
  }

  /// 更新操作进度
  ///
  /// # 参数
  /// - `operation_id`: 操作ID
  /// - `processed_bytes`: 已处理字节数
  pub async fn update_progress(&self, operation_id: &str, processed_bytes: u64) -> Result<(), BinaryStorageError> {
    let mut operations = self.operations.write().await;
    if let Some(progress) = operations.get_mut(operation_id) {
      progress.update_progress(processed_bytes);
      Ok(())
    } else {
      Err(BinaryStorageError::metrics(format!("操作不存在: {}", operation_id)))
    }
  }

  /// 完成操作
  ///
  /// # 参数
  /// - `operation_id`: 操作ID
  /// - `success`: 是否成功
  /// - `bytes_processed`: 处理的字节数
  pub async fn complete_operation(
    &self,
    operation_id: &str,
    success: bool,
    bytes_processed: u64,
  ) -> Result<(), BinaryStorageError> {
    let mut operations = self.operations.write().await;
    if let Some(progress) = operations.get_mut(operation_id) {
      progress.mark_completed(success);

      // 更新统计
      let mut stats = self.stats.write().await;
      stats.total_operations += 1; // 在这里增加总操作数
      if success {
        stats.successful_operations += 1;
        stats.total_bytes_processed += bytes_processed;
      } else {
        stats.failed_operations += 1;
      }

      Ok(())
    } else {
      Err(BinaryStorageError::metrics(format!("操作不存在: {}", operation_id)))
    }
  }

  /// 设置操作错误
  ///
  /// # 参数
  /// - `operation_id`: 操作ID
  /// - `error`: 错误信息
  pub async fn set_operation_error(&self, operation_id: &str, error: String) -> Result<(), BinaryStorageError> {
    let mut operations = self.operations.write().await;
    if let Some(progress) = operations.get_mut(operation_id) {
      progress.set_error(error);

      // 更新统计
      let mut stats = self.stats.write().await;
      stats.total_operations += 1; // 在这里增加总操作数
      stats.failed_operations += 1;

      Ok(())
    } else {
      Err(BinaryStorageError::metrics(format!("操作不存在: {}", operation_id)))
    }
  }

  /// 获取操作进度
  ///
  /// # 参数
  /// - `operation_id`: 操作ID
  ///
  /// # 返回
  /// - `Option<OperationProgress>`: 操作进度，如果不存在则返回None
  pub async fn get_progress(&self, operation_id: &str) -> Option<OperationProgress> {
    let operations = self.operations.read().await;
    operations.get(operation_id).cloned()
  }

  /// 获取所有当前操作
  ///
  /// # 返回
  /// - `Vec<OperationProgress>`: 所有当前操作的进度
  pub async fn get_all_operations(&self) -> Vec<OperationProgress> {
    let operations = self.operations.read().await;
    operations.values().cloned().collect()
  }

  /// 获取指定类型的操作
  ///
  /// # 参数
  /// - `operation_type`: 操作类型
  ///
  /// # 返回
  /// - `Vec<OperationProgress>`: 指定类型的操作进度
  pub async fn get_operations_by_type(&self, operation_type: &str) -> Vec<OperationProgress> {
    let operations = self.operations.read().await;
    operations.values().filter(|progress| progress.operation_type == operation_type).cloned().collect()
  }

  /// 获取统计信息
  ///
  /// # 返回
  /// - `BasicStats`: 统计信息
  pub async fn get_stats(&self) -> BasicStats {
    let stats = self.stats.read().await;
    stats.clone()
  }

  /// 清理已完成的操作（保留最近的一些记录）
  ///
  /// # 参数
  /// - `keep_count`: 保留的已完成操作数量
  pub async fn cleanup_completed_operations(&self, keep_count: usize) {
    // First, collect the operation IDs to remove using a read lock
    let operation_ids_to_remove = {
      let operations = self.operations.read().await;

      // 收集已完成的操作
      let mut completed_operations: Vec<_> = operations.iter().filter(|(_, progress)| progress.is_completed).collect();

      // 按开始时间排序，保留最新的keep_count个
      completed_operations.sort_by(|a, b| b.1.start_time.cmp(&a.1.start_time));

      // 收集需要删除的操作ID
      completed_operations
        .iter()
        .skip(keep_count)
        .map(|(operation_id, _)| (*operation_id).clone())
        .collect::<Vec<_>>()
    };

    // Now remove the operations using a write lock
    let mut operations = self.operations.write().await;
    for operation_id in operation_ids_to_remove {
      operations.remove(&operation_id);
    }
  }

  /// 清理所有操作
  pub async fn clear_all_operations(&self) {
    let mut operations = self.operations.write().await;
    operations.clear();
  }

  /// 重置统计信息
  pub async fn reset_stats(&self) {
    let mut stats = self.stats.write().await;
    *stats = BasicStats::default();
  }
}

impl Default for BasicMetricsCollector {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_operation_progress() {
    let mut progress = OperationProgress {
      operation_id: "test".to_string(),
      operation_type: "store".to_string(),
      total_bytes: 100,
      processed_bytes: 0,
      start_time: Instant::now(),
      is_completed: false,
      error: None,
    };

    // 测试初始状态
    assert_eq!(progress.completion_percentage(), 0.0);
    assert!(progress.estimated_remaining_time().is_none());

    // 测试更新进度
    progress.update_progress(50);
    assert_eq!(progress.completion_percentage(), 50.0);

    // 测试完成
    progress.mark_completed(true);
    assert!(progress.is_completed);
    assert!(progress.error.is_none());

    // 测试错误
    let mut progress = OperationProgress {
      operation_id: "test".to_string(),
      operation_type: "store".to_string(),
      total_bytes: 100,
      processed_bytes: 0,
      start_time: Instant::now(),
      is_completed: false,
      error: None,
    };

    progress.set_error("测试错误".to_string());
    assert!(progress.is_completed);
    assert_eq!(progress.error, Some("测试错误".to_string()));
  }

  #[tokio::test]
  async fn test_metrics_collector() {
    let collector = BasicMetricsCollector::new();

    // 重置统计信息以确保从零开始
    collector.reset_stats().await;

    // 测试开始操作
    let operation_id = collector.start_operation_async("store", 100).await;
    assert!(!operation_id.is_empty());

    // 测试获取进度
    let progress = collector.get_progress(&operation_id).await;
    assert!(progress.is_some());
    assert_eq!(progress.unwrap().total_bytes, 100);

    // 测试更新进度
    collector.update_progress(&operation_id, 50).await.unwrap();
    let progress = collector.get_progress(&operation_id).await.unwrap();
    assert_eq!(progress.processed_bytes, 50);
    assert_eq!(progress.completion_percentage(), 50.0);

    // 测试完成操作
    collector.complete_operation(&operation_id, true, 100).await.unwrap();
    let progress = collector.get_progress(&operation_id).await.unwrap();
    assert!(progress.is_completed);

    // 测试统计信息
    let stats = collector.get_stats().await;
    assert_eq!(stats.total_operations, 1);
    assert_eq!(stats.successful_operations, 1);
    assert_eq!(stats.failed_operations, 0);
    assert_eq!(stats.total_bytes_processed, 100);

    // 测试失败操作
    let operation_id2 = collector.start_operation_async("retrieve", 50).await;
    collector.complete_operation(&operation_id2, false, 0).await.unwrap();

    let stats = collector.get_stats().await;
    assert_eq!(stats.total_operations, 2);
    assert_eq!(stats.successful_operations, 1);
    assert_eq!(stats.failed_operations, 1);
    assert_eq!(stats.total_bytes_processed, 100);
  }

  #[tokio::test]
  async fn test_cleanup_operations() {
    let collector = BasicMetricsCollector::new();

    // 创建几个操作
    let id1 = collector.start_operation_async("store", 100).await;
    let id2 = collector.start_operation_async("retrieve", 50).await;
    let id3 = collector.start_operation_async("delete", 0).await;

    // 完成所有操作
    collector.complete_operation(&id1, true, 100).await.unwrap();
    collector.complete_operation(&id2, true, 50).await.unwrap();
    collector.complete_operation(&id3, false, 0).await.unwrap();

    // 验证所有操作都存在
    let operations = collector.get_all_operations().await;
    assert_eq!(operations.len(), 3);

    // 清理操作，只保留1个
    collector.cleanup_completed_operations(1).await;

    // 验证只有1个操作保留
    let operations = collector.get_all_operations().await;
    assert_eq!(operations.len(), 1);
  }
}
