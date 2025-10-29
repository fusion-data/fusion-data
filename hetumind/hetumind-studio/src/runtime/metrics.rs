use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use fusion_common::ahash::HashMap;
use fusion_common::ahash::HashMapExt;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use hetumind_core::{
  workflow::MonitoringConfig,
  workflow::{ExecutionId, ExecutionMetrics},
};

/// 执行指标收集器
#[derive(Debug)]
pub struct ExecutionMetricsCollector {
  config: MonitoringConfig,
  metrics: Arc<RwLock<HashMap<ExecutionId, ExecutionMetrics>>>,
  sampling_rates: HashMap<String, f64>,
}

impl Clone for ExecutionMetricsCollector {
  fn clone(&self) -> Self {
    Self { config: self.config.clone(), metrics: self.metrics.clone(), sampling_rates: self.sampling_rates.clone() }
  }
}

impl ExecutionMetricsCollector {
  pub fn new(config: MonitoringConfig) -> Self {
    Self { config, metrics: Arc::new(RwLock::new(HashMap::new())), sampling_rates: HashMap::new() }
  }

  /// 记录执行指标
  pub async fn record_metrics(&self, metrics: ExecutionMetrics) {
    if !self.config.enable_metrics {
      return;
    }

    // 基于采样率决定是否记录
    if self.should_sample(&metrics.execution_id) {
      let mut metrics_map = self.metrics.write().await;
      metrics_map.insert(metrics.execution_id, metrics);
    }
  }

  /// 获取执行指标
  pub async fn get_metrics(&self, execution_id: &ExecutionId) -> Option<ExecutionMetrics> {
    let metrics_map = self.metrics.read().await;
    metrics_map.get(execution_id).cloned()
  }

  /// 获取所有指标
  pub async fn get_all_metrics(&self) -> Vec<ExecutionMetrics> {
    let metrics_map = self.metrics.read().await;
    metrics_map.values().cloned().collect()
  }

  /// 清理过期指标
  pub async fn cleanup_old_metrics(&self, older_than: Duration) {
    let cutoff = (chrono::Utc::now() - chrono::Duration::from_std(older_than).unwrap_or(chrono::Duration::zero()))
      .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()));

    let mut metrics_map = self.metrics.write().await;
    metrics_map.retain(|_, metrics| {
      // 假设指标中有开始时间信息，如果没有需要修改这个逻辑
      // 这里基于execution_id的UUID时间戳来估算
      if let Some(timestamp) = self.extract_timestamp_from_id(&metrics.execution_id) {
        timestamp > cutoff
      } else {
        true // 如果无法提取时间戳，保留该指标
      }
    });
  }

  /// 计算统计信息
  pub async fn get_statistics(&self) -> ExecutionStatistics {
    let metrics_map = self.metrics.read().await;
    let metrics: Vec<&ExecutionMetrics> = metrics_map.values().collect();

    if metrics.is_empty() {
      return ExecutionStatistics::default();
    }

    let total_executions = metrics.len() as u64;
    let successful_executions = metrics.iter().filter(|m| m.nodes_succeeded > 0).count() as u64;
    let failed_executions = metrics.iter().filter(|m| m.nodes_failed > 0).count() as u64;

    let avg_duration = metrics.iter().map(|m| m.duration_ms).sum::<u64>() / total_executions;

    let avg_memory_usage = metrics.iter().map(|m| m.memory_usage_mb).sum::<f64>() / total_executions as f64;

    let avg_cache_hit_rate = metrics.iter().map(|m| m.cache_hit_rate).sum::<f64>() / total_executions as f64;

    let avg_cpu_usage = metrics.iter().map(|m| m.cpu_usage_percent).sum::<f64>() / total_executions as f64;

    let total_retry_count = metrics.iter().map(|m| m.retry_count).sum::<u32>();

    ExecutionStatistics {
      total_executions,
      successful_executions,
      failed_executions,
      success_rate: if total_executions > 0 { successful_executions as f64 / total_executions as f64 } else { 0.0 },
      average_duration_ms: avg_duration,
      average_memory_usage_mb: avg_memory_usage,
      average_cache_hit_rate: avg_cache_hit_rate,
      average_cpu_usage_percent: avg_cpu_usage,
      total_retry_count,
      uptime_seconds: chrono::Utc::now().timestamp() as u64,
    }
  }

  /// 从执行ID中提取时间戳（UUID v1/v6时间戳）
  fn extract_timestamp_from_id(&self, _execution_id: &ExecutionId) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    // 简化实现，直接返回当前时间作为替代
    // 在实际项目中，这里应该基于execution_id或其他时间戳信息
    // 使用UTC+0时区
    let offset = chrono::FixedOffset::east_opt(0).unwrap();
    Some(chrono::Utc::now().with_timezone(&offset))
  }

  /// 采样率控制逻辑
  fn should_sample(&self, execution_id: &ExecutionId) -> bool {
    use rand::Rng;

    let sample_rate = self
      .sampling_rates
      .get(&execution_id.to_string())
      .copied()
      .unwrap_or(self.config.metrics_sample_rate);

    if sample_rate >= 1.0 {
      return true;
    }

    let mut rng = rand::rng();
    rng.random::<f64>() < sample_rate
  }

  /// 设置特定执行ID的采样率
  pub fn set_sampling_rate(&mut self, execution_id: ExecutionId, rate: f64) {
    self.sampling_rates.insert(execution_id.to_string(), rate);
  }

  /// 获取指标数量
  pub async fn metrics_count(&self) -> usize {
    let metrics_map = self.metrics.read().await;
    metrics_map.len()
  }

  /// 清空所有指标
  pub async fn clear_metrics(&self) {
    let mut metrics_map = self.metrics.write().await;
    metrics_map.clear();
  }
}

/// 执行统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
  pub total_executions: u64,
  pub successful_executions: u64,
  pub failed_executions: u64,
  pub success_rate: f64,
  pub average_duration_ms: u64,
  pub average_memory_usage_mb: f64,
  pub average_cache_hit_rate: f64,
  pub average_cpu_usage_percent: f64,
  pub total_retry_count: u32,
  pub uptime_seconds: u64,
}

impl Default for ExecutionStatistics {
  fn default() -> Self {
    Self {
      total_executions: 0,
      successful_executions: 0,
      failed_executions: 0,
      success_rate: 0.0,
      average_duration_ms: 0,
      average_memory_usage_mb: 0.0,
      average_cache_hit_rate: 0.0,
      average_cpu_usage_percent: 0.0,
      total_retry_count: 0,
      uptime_seconds: 0,
    }
  }
}

/// 实时性能监控器
#[derive(Clone)]
pub struct PerformanceMonitor {
  metrics_collector: Arc<ExecutionMetricsCollector>,
  alert_thresholds: AlertThresholds,
  alert_handlers: Vec<Arc<dyn AlertHandler>>,
}

impl std::fmt::Debug for PerformanceMonitor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("PerformanceMonitor")
      .field("metrics_collector", &"<ExecutionMetricsCollector>")
      .field("alert_thresholds", &self.alert_thresholds)
      .field("alert_handlers", &self.alert_handlers.len())
      .finish()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
  pub max_execution_time_ms: u64,
  pub max_memory_usage_mb: f64,
  pub max_error_rate: f64,
  pub min_cache_hit_rate: f64,
  pub max_cpu_usage_percent: f64,
  pub max_retry_count: u32,
}

impl Default for AlertThresholds {
  fn default() -> Self {
    Self {
      max_execution_time_ms: 300000, // 5 minutes
      max_memory_usage_mb: 1024.0,   // 1GB
      max_error_rate: 0.1,           // 10%
      min_cache_hit_rate: 0.8,       // 80%
      max_cpu_usage_percent: 80.0,   // 80%
      max_retry_count: 5,            // 5 retries
    }
  }
}

#[async_trait]
pub trait AlertHandler: Send + Sync {
  async fn handle_alert(&self, alert: PerformanceAlert) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
  pub alert_type: AlertType,
  pub severity: AlertSeverity,
  pub message: String,
  pub metrics: ExecutionMetrics,
  pub timestamp: chrono::DateTime<chrono::FixedOffset>,
  pub execution_id: ExecutionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
  ExecutionTimeout,
  MemoryUsageExceeded,
  HighErrorRate,
  LowCacheHitRate,
  HighCpuUsage,
  HighRetryCount,
  NodeExecutionFailure,
  WorkflowExecutionFailure,
}

impl std::fmt::Display for AlertType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AlertType::ExecutionTimeout => write!(f, "ExecutionTimeout"),
      AlertType::MemoryUsageExceeded => write!(f, "MemoryUsageExceeded"),
      AlertType::HighErrorRate => write!(f, "HighErrorRate"),
      AlertType::LowCacheHitRate => write!(f, "LowCacheHitRate"),
      AlertType::HighCpuUsage => write!(f, "HighCpuUsage"),
      AlertType::HighRetryCount => write!(f, "HighRetryCount"),
      AlertType::NodeExecutionFailure => write!(f, "NodeExecutionFailure"),
      AlertType::WorkflowExecutionFailure => write!(f, "WorkflowExecutionFailure"),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
  Info,
  Warning,
  Error,
  Critical,
}

impl PerformanceMonitor {
  pub fn new(metrics_collector: Arc<ExecutionMetricsCollector>) -> Self {
    Self { metrics_collector, alert_thresholds: AlertThresholds::default(), alert_handlers: Vec::new() }
  }

  /// 设置告警阈值
  pub fn set_alert_thresholds(&mut self, thresholds: AlertThresholds) {
    self.alert_thresholds = thresholds;
  }

  /// 添加告警处理器
  pub fn add_alert_handler<H: AlertHandler + 'static>(&mut self, handler: H) {
    self.alert_handlers.push(Arc::new(handler));
  }

  /// 监控执行指标
  pub async fn monitor_execution(&self, metrics: &ExecutionMetrics) {
    // 检查执行时间
    if metrics.duration_ms > self.alert_thresholds.max_execution_time_ms {
      self
        .trigger_alert(PerformanceAlert {
          alert_type: AlertType::ExecutionTimeout,
          severity: AlertSeverity::Warning,
          message: format!("Execution time exceeded threshold: {}ms", metrics.duration_ms),
          metrics: metrics.clone(),
          timestamp: chrono::Utc::now().with_timezone(
            &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
          ),
          execution_id: metrics.execution_id,
        })
        .await;
    }

    // 检查内存使用
    if metrics.memory_usage_mb > self.alert_thresholds.max_memory_usage_mb {
      self
        .trigger_alert(PerformanceAlert {
          alert_type: AlertType::MemoryUsageExceeded,
          severity: AlertSeverity::Warning,
          message: format!("Memory usage exceeded threshold: {}MB", metrics.memory_usage_mb),
          metrics: metrics.clone(),
          timestamp: chrono::Utc::now().with_timezone(
            &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
          ),
          execution_id: metrics.execution_id,
        })
        .await;
    }

    // 检查错误率
    if metrics.nodes_executed > 0 {
      let error_rate = metrics.nodes_failed as f64 / metrics.nodes_executed as f64;
      if error_rate > self.alert_thresholds.max_error_rate {
        self
          .trigger_alert(PerformanceAlert {
            alert_type: AlertType::HighErrorRate,
            severity: AlertSeverity::Error,
            message: format!("High error rate: {:.2}%", error_rate * 100.0),
            metrics: metrics.clone(),
            timestamp: chrono::Utc::now().with_timezone(
              &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
            ),
            execution_id: metrics.execution_id,
          })
          .await;
      }
    }

    // 检查缓存命中率
    if metrics.cache_hit_rate < self.alert_thresholds.min_cache_hit_rate {
      self
        .trigger_alert(PerformanceAlert {
          alert_type: AlertType::LowCacheHitRate,
          severity: AlertSeverity::Info,
          message: format!("Low cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0),
          metrics: metrics.clone(),
          timestamp: chrono::Utc::now().with_timezone(
            &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
          ),
          execution_id: metrics.execution_id,
        })
        .await;
    }

    // 检查CPU使用率
    if metrics.cpu_usage_percent > self.alert_thresholds.max_cpu_usage_percent {
      self
        .trigger_alert(PerformanceAlert {
          alert_type: AlertType::HighCpuUsage,
          severity: AlertSeverity::Warning,
          message: format!("High CPU usage: {:.2}%", metrics.cpu_usage_percent),
          metrics: metrics.clone(),
          timestamp: chrono::Utc::now().with_timezone(
            &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
          ),
          execution_id: metrics.execution_id,
        })
        .await;
    }

    // 检查重试次数
    if metrics.retry_count > self.alert_thresholds.max_retry_count {
      self
        .trigger_alert(PerformanceAlert {
          alert_type: AlertType::HighRetryCount,
          severity: AlertSeverity::Warning,
          message: format!("High retry count: {}", metrics.retry_count),
          metrics: metrics.clone(),
          timestamp: chrono::Utc::now().with_timezone(
            &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
          ),
          execution_id: metrics.execution_id,
        })
        .await;
    }

    // 检查执行失败
    if metrics.nodes_failed > 0 {
      self
        .trigger_alert(PerformanceAlert {
          alert_type: AlertType::NodeExecutionFailure,
          severity: AlertSeverity::Error,
          message: format!("Node execution failures: {}", metrics.nodes_failed),
          metrics: metrics.clone(),
          timestamp: chrono::Utc::now().with_timezone(
            &chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap()),
          ),
          execution_id: metrics.execution_id,
        })
        .await;
    }
  }

  /// 触发告警
  async fn trigger_alert(&self, alert: PerformanceAlert) {
    for handler in &self.alert_handlers {
      if let Err(e) = handler.handle_alert(alert.clone()).await {
        error!("Failed to handle alert: {}", e);
      }
    }
  }

  /// 启动后台监控任务
  pub async fn start_background_monitoring(&self) {
    let collector = self.metrics_collector.clone();
    let monitor = self.clone();

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_secs(60)); // 每分钟检查一次

      loop {
        interval.tick().await;

        // 获取最近的指标进行监控
        let metrics = collector.get_all_metrics().await;
        for metric in metrics {
          monitor.monitor_execution(&metric).await;
        }
      }
    });
  }

  /// 获取实时监控摘要
  pub async fn get_monitoring_summary(&self) -> MonitoringSummary {
    let stats = self.metrics_collector.get_statistics().await;

    MonitoringSummary {
      statistics: stats,
      alert_thresholds: self.alert_thresholds.clone(),
      active_alerts_count: self.alert_handlers.len() as u32,
      last_check_time: chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap())),
    }
  }
}

/// 监控摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSummary {
  pub statistics: ExecutionStatistics,
  pub alert_thresholds: AlertThresholds,
  pub active_alerts_count: u32,
  pub last_check_time: chrono::DateTime<chrono::FixedOffset>,
}

/// 默认的日志告警处理器
pub struct LogAlertHandler;

#[async_trait]
impl AlertHandler for LogAlertHandler {
  async fn handle_alert(&self, alert: PerformanceAlert) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match alert.severity {
      AlertSeverity::Info => {
        info!("Performance Alert [{}]: {} for execution {}", alert.alert_type, alert.message, alert.execution_id);
      }
      AlertSeverity::Warning => {
        warn!("Performance Alert [{}]: {} for execution {}", alert.alert_type, alert.message, alert.execution_id);
      }
      AlertSeverity::Error => {
        error!("Performance Alert [{}]: {} for execution {}", alert.alert_type, alert.message, alert.execution_id);
      }
      AlertSeverity::Critical => {
        error!(
          "CRITICAL Performance Alert [{}]: {} for execution {}",
          alert.alert_type, alert.message, alert.execution_id
        );
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hetumind_core::workflow::ExecutionId;

  #[tokio::test]
  async fn test_metrics_collection() {
    let config = MonitoringConfig {
      enable_metrics: true,
      enable_tracing: false,
      metrics_sample_rate: 1.0,
      tracing_sample_rate: 0.0,
    };

    let collector = ExecutionMetricsCollector::new(config);

    let metrics = ExecutionMetrics {
      execution_id: ExecutionId::now_v7(),
      duration_ms: 1000,
      nodes_executed: 5,
      nodes_succeeded: 5,
      nodes_failed: 0,
      memory_usage_mb: 100.0,
      cpu_usage_percent: 50.0,
      cache_hit_rate: 0.8,
      retry_count: 0,
    };

    collector.record_metrics(metrics.clone()).await;

    let retrieved_metrics = collector.get_metrics(&metrics.execution_id).await;
    assert!(retrieved_metrics.is_some());

    let stats = collector.get_statistics().await;
    assert_eq!(stats.total_executions, 1);
    assert_eq!(stats.successful_executions, 1);
    assert_eq!(stats.failed_executions, 0);
  }

  #[tokio::test]
  async fn test_sampling_rate() {
    let config = MonitoringConfig {
      enable_metrics: true,
      enable_tracing: false,
      metrics_sample_rate: 0.0, // 0% 采样率
      tracing_sample_rate: 0.0,
    };

    let collector = ExecutionMetricsCollector::new(config);

    let metrics = ExecutionMetrics {
      execution_id: ExecutionId::now_v7(),
      duration_ms: 1000,
      nodes_executed: 5,
      nodes_succeeded: 5,
      nodes_failed: 0,
      memory_usage_mb: 100.0,
      cpu_usage_percent: 50.0,
      cache_hit_rate: 0.8,
      retry_count: 0,
    };

    collector.record_metrics(metrics.clone()).await;

    let retrieved_metrics = collector.get_metrics(&metrics.execution_id).await;
    assert!(retrieved_metrics.is_none()); // 应该被采样过滤掉
  }

  #[tokio::test]
  async fn test_performance_monitoring() {
    let config = MonitoringConfig {
      enable_metrics: true,
      enable_tracing: false,
      metrics_sample_rate: 1.0,
      tracing_sample_rate: 0.0,
    };

    let collector = Arc::new(ExecutionMetricsCollector::new(config));
    let monitor = PerformanceMonitor::new(collector.clone());

    let metrics = ExecutionMetrics {
      execution_id: ExecutionId::now_v7(),
      duration_ms: 400000, // 超过默认阈值
      nodes_executed: 5,
      nodes_succeeded: 3,
      nodes_failed: 2,
      memory_usage_mb: 2000.0, // 超过默认阈值
      cpu_usage_percent: 90.0, // 超过默认阈值
      cache_hit_rate: 0.5,     // 低于默认阈值
      retry_count: 10,         // 超过默认阈值
    };

    // 添加日志告警处理器
    let mut monitor_clone = monitor.clone();
    monitor_clone.add_alert_handler(LogAlertHandler);

    // 监控指标（应该触发多个告警）
    monitor_clone.monitor_execution(&metrics).await;
  }
}
