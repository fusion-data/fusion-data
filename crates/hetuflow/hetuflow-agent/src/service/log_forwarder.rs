use std::collections::VecDeque;
use std::sync::Arc;

use fusion_common::time::{now_offset, now_epoch_millis, OffsetDateTime};
use hetuflow_core::protocol::{LogMessage, LogBatch, LogType};
use hetuflow_core::protocol::WebSocketEvent;
use hetuflow_core::types::EventKind;
use log::{debug, error, warn};
use serde_json::json;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Instant};
use uuid;

/// 日志转发统计信息
#[derive(Debug, Default, Clone)]
pub struct LogForwardStats {
  /// 转发成功数量
  forwarded_count: u64,
  /// 转发失败数量
  failed_count: u64,
  /// 最后转发时间
  last_forward_time: Option<OffsetDateTime>,
  /// 连续失败次数
  consecutive_failures: u32,
  /// 平均转发延迟（毫秒）
  avg_forward_latency_ms: f64,
  /// 总处理的日志条数
  total_logs_processed: u64,
  /// 缓冲区溢出次数
  buffer_overflow_count: u64,
}

use crate::setting::LogForwardingConfig;

/// 日志转发器，负责收集、批量处理和转发任务日志
#[derive(Debug)]
pub struct LogForwarder {
  /// 配置
  config: Arc<LogForwardingConfig>,
  /// 日志缓冲区
  buffer: Arc<Mutex<VecDeque<LogMessage>>>,
  /// WebSocket事件发送器
  event_sender: Option<mpsc::UnboundedSender<WebSocketEvent>>,
  /// 停止信号
  shutdown_tx: Option<mpsc::UnboundedSender<()>>,
  /// 性能统计信息
  stats: Arc<Mutex<LogForwardStats>>,
}

impl LogForwarder {
  /// 创建新的日志转发器
  pub fn new(
    config: Arc<LogForwardingConfig>,
    event_sender: Option<mpsc::UnboundedSender<WebSocketEvent>>,
  ) -> Self {
    Self {
      config,
      buffer: Arc::new(Mutex::new(VecDeque::new())),
      event_sender,
      shutdown_tx: None,
      stats: Arc::new(Mutex::new(LogForwardStats::default())),
    }
  }

  /// 启动日志转发器
  pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !self.config.enabled {
      debug!("日志转发已禁用，跳过启动");
      return Ok(());
    }

    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
    self.shutdown_tx = Some(shutdown_tx);

    let config = self.config.clone();
    let buffer = self.buffer.clone();
    let event_sender = self.event_sender.clone();

    let stats = self.stats.clone();
    
    // 启动定时刷新任务
    tokio::spawn(async move {
      let mut flush_interval = interval(config.flush_interval);
      let mut retry_count = 0;
      let mut _last_flush = Instant::now();

      loop {
        tokio::select! {
          _ = flush_interval.tick() => {
            if let Err(e) = Self::flush_buffer(&config, &buffer, &event_sender, &stats).await {
              retry_count += 1;
              error!("日志刷新失败 (重试 {}/{}): {}", retry_count, config.max_retries, e);
              
              if retry_count >= config.max_retries {
                warn!("达到最大重试次数，跳过本次刷新");
                retry_count = 0;
              } else {
                // 等待重试间隔
                tokio::time::sleep(config.retry_interval).await;
                continue;
              }
            } else {
              retry_count = 0;
            }
            _last_flush = Instant::now();
          }
          _ = shutdown_rx.recv() => {
            debug!("收到停止信号，执行最后一次刷新");
            let _ = Self::flush_buffer(&config, &buffer, &event_sender, &stats).await;
            break;
          }
        }
      }
    });

    debug!("日志转发器已启动");
    Ok(())
  }

  /// 停止日志转发器
  pub async fn stop(&mut self) {
    if let Some(shutdown_tx) = self.shutdown_tx.take() {
      let _ = shutdown_tx.send(());
      debug!("日志转发器停止信号已发送");
    }
  }

  /// 添加日志消息到缓冲区
  pub async fn add_log(
    &self,
    task_id: String,
    instance_id: String,
    log_type: LogType,
    content: String,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !self.config.enabled {
      return Ok(());
    }

    // 检查内容长度限制
    let truncated_content = if content.len() > self.config.buffer_size {
      warn!("日志内容超过最大长度限制，将被截断: {} > {}", content.len(), self.config.buffer_size);
      content.chars().take(self.config.buffer_size).collect()
    } else {
      content
    };

    let log_message = LogMessage {
      task_id: uuid::Uuid::parse_str(&task_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
      task_instance_id: uuid::Uuid::parse_str(&instance_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
      sequence: 0, // TODO: 实现序列号管理
      log_type,
      content: truncated_content,
      timestamp: now_epoch_millis(),
      agent_id: "agent".to_string(), // TODO: 从配置获取
      process_id: None,
      level: Some("INFO".to_string()),
      source: Some("process".to_string()),
    };

    let mut buffer = self.buffer.lock().await;
    buffer.push_back(log_message);

    // 检查缓冲区是否溢出
    if buffer.len() > self.config.buffer_size {
      let mut stats_guard = self.stats.lock().await;
      stats_guard.buffer_overflow_count += 1;
      drop(stats_guard);
      warn!("缓冲区溢出，当前大小: {}", buffer.len());
    }

    // 检查是否需要立即刷新
    if buffer.len() >= self.config.batch_size {
      drop(buffer); // 释放锁
      if let Err(e) = Self::flush_buffer(&self.config, &self.buffer, &self.event_sender, &self.stats).await {
        error!("立即刷新缓冲区失败: {}", e);
      }
    }

    Ok(())
  }

  /// 刷新缓冲区，发送日志批次（带重试和性能监控）
  async fn flush_buffer(
    config: &LogForwardingConfig,
    buffer: &Arc<Mutex<VecDeque<LogMessage>>>,
    event_sender: &Option<mpsc::UnboundedSender<WebSocketEvent>>,
    stats: &Arc<Mutex<LogForwardStats>>,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();
    let mut buffer_guard = buffer.lock().await;
    if buffer_guard.is_empty() {
      return Ok(());
    }

    // 取出批次数据
    let batch_size = std::cmp::min(buffer_guard.len(), config.batch_size);
    let mut logs = Vec::with_capacity(batch_size);
    for _ in 0..batch_size {
      if let Some(log) = buffer_guard.pop_front() {
        logs.push(log);
      }
    }
    drop(buffer_guard); // 释放锁

    if logs.is_empty() {
      return Ok(());
    }

    let log_count = logs.len();

    // 创建日志批次
    let log_batch = LogBatch {
      batch_id: uuid::Uuid::new_v4(),
      messages: logs,
      batch_timestamp: now_epoch_millis(),
      compressed: config.enable_compression,
    };

    // 发送WebSocket事件（带重试机制）
    if let Some(sender) = event_sender {
      let mut last_error = None;
      
      for attempt in 0..=config.max_retries {
        let event = WebSocketEvent::new(
          EventKind::TaskLog,
          json!({
            "batch": log_batch
          }),
        );

        match sender.send(event) {
          Ok(_) => {
            let latency = start_time.elapsed().as_millis() as f64;
            
            // 更新成功统计信息
            {
              let mut stats_guard = stats.lock().await;
              stats_guard.forwarded_count += log_count as u64;
              stats_guard.total_logs_processed += log_count as u64;
              stats_guard.last_forward_time = Some(now_offset());
              stats_guard.consecutive_failures = 0;
              
              // 更新平均延迟（指数移动平均）
              if stats_guard.avg_forward_latency_ms == 0.0 {
                stats_guard.avg_forward_latency_ms = latency;
              } else {
                stats_guard.avg_forward_latency_ms = stats_guard.avg_forward_latency_ms * 0.9 + latency * 0.1;
              }
            }

            debug!("已发送日志批次，包含 {} 条日志，耗时 {}ms (尝试 {})", log_count, latency as u64, attempt + 1);
            return Ok(());
          }
          Err(e) => {
            last_error = Some(format!("发送日志批次事件失败: {}", e));
            
            // 更新失败统计信息
            {
              let mut stats_guard = stats.lock().await;
              stats_guard.failed_count += 1;
              stats_guard.consecutive_failures += 1;
            }
            
            if attempt < config.max_retries {
              warn!("日志发送失败 (尝试 {}), {}ms后重试: {:?}", 
                    attempt + 1, config.retry_interval.as_millis(), last_error);
              tokio::time::sleep(config.retry_interval).await;
            }
          }
        }
      }
      
      // 所有重试都失败了
      let error_msg = format!("日志批次发送失败，已重试 {} 次: {:?}", 
                             config.max_retries + 1, last_error);
      error!("{}", error_msg);
      return Err(error_msg.into());
    }

    Ok(())
  }

  /// 获取缓冲区状态
  pub async fn get_buffer_status(&self) -> (usize, usize) {
    let buffer = self.buffer.lock().await;
    (buffer.len(), self.config.batch_size)
  }

  /// 获取性能统计信息
  pub async fn get_stats(&self) -> LogForwardStats {
    let stats = self.stats.lock().await;
    stats.clone()
  }

  /// 重置统计信息
  pub async fn reset_stats(&self) {
    let mut stats = self.stats.lock().await;
    *stats = LogForwardStats::default();
  }
}

impl Drop for LogForwarder {
  fn drop(&mut self) {
    if let Some(shutdown_tx) = self.shutdown_tx.take() {
      let _ = shutdown_tx.send(());
    }
  }
}

#[cfg(test)]
  mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::sync::mpsc;

  #[tokio::test]
  async fn test_log_forwarder_creation() {
    let config = Arc::new(LogForwardingConfig {
      enabled: true,
      buffer_size: 1024,
      batch_size: 10,
      flush_interval: Duration::from_secs(1),
      max_retries: 3,
      retry_interval: Duration::from_secs(1),
      enable_compression: false,
    });

    let (tx, _rx) = mpsc::unbounded_channel();
    let forwarder = LogForwarder::new(config, Some(tx));

    assert!(forwarder.config.enabled);
    assert_eq!(forwarder.config.batch_size, 10);
  }

  #[tokio::test]
  async fn test_add_log() {
    let config = Arc::new(LogForwardingConfig {
      enabled: true,
      buffer_size: 1024,
      batch_size: 10,
      flush_interval: Duration::from_secs(1),
      max_retries: 3,
      retry_interval: Duration::from_secs(1),
      enable_compression: false,
    });

    let (tx, _rx) = mpsc::unbounded_channel();
    let forwarder = LogForwarder::new(config, Some(tx));

    let result = forwarder.add_log(
      "task_1".to_string(),
      "instance_1".to_string(),
      LogType::Stdout,
      "test log message".to_string(),
    ).await;

    assert!(result.is_ok());
    
    let (buffer_len, _) = forwarder.get_buffer_status().await;
    assert_eq!(buffer_len, 1);
  }
}