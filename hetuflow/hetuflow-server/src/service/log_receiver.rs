use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use fusion_common::time::{OffsetDateTime, now_offset};
use fusion_core::DataError;
use hetuflow_core::protocol::{LogBatch, LogMessage};
// use hetuflow_core::protocol::websocket::{WebSocketEvent, WebSocketCommand};
// use hetuflow_core::types::{EventKind, CommandKind};
use fusion_common::time::now_epoch_millis;
use log::{debug, error, info};
use serde_json::Value;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::interval;

use crate::model::GatewayCommandRequest;
use crate::setting::{TaskLogConfig, WebSocketLogConfig};

/// 日志接收器，负责接收Agent转发的日志并存储到文件
#[derive(Debug)]
pub struct LogReceiver {
  /// 任务日志配置
  config: Arc<TaskLogConfig>,
  /// WebSocket日志配置
  websocket_config: Arc<WebSocketLogConfig>,
  /// 文件写入器缓存
  file_writers: Arc<RwLock<HashMap<String, Arc<Mutex<BufWriter<File>>>>>>,
  /// 日志统计信息
  stats: Arc<RwLock<LogStats>>,
  /// 停止信号
  shutdown_tx: Option<mpsc::UnboundedSender<()>>,
  /// Gateway命令发送器
  gateway_tx: Option<mpsc::UnboundedSender<GatewayCommandRequest>>,
}

/// 日志统计信息
#[derive(Debug, Default)]
struct LogStats {
  /// 接收的日志总数
  total_received: u64,
  /// 写入的日志总数
  total_written: u64,
  /// 错误计数
  error_count: u64,
  /// 最后更新时间
  last_update: Option<OffsetDateTime>,
  /// WebSocket转发成功数
  websocket_forwarded: u64,
  /// WebSocket转发失败数
  websocket_failed: u64,
  /// 平均处理延迟（毫秒）
  avg_processing_latency_ms: f64,
  /// 文件写入失败数
  file_write_failed: u64,
  /// 连续错误数
  consecutive_errors: u32,
}

impl LogReceiver {
  /// 创建新的日志接收器
  pub fn new(config: Arc<TaskLogConfig>, websocket_config: Arc<WebSocketLogConfig>) -> Self {
    Self {
      config,
      websocket_config,
      file_writers: Arc::new(RwLock::new(HashMap::new())),
      stats: Arc::new(RwLock::new(LogStats::default())),
      shutdown_tx: None,
      gateway_tx: None,
    }
  }

  /// 设置Gateway命令发送器
  pub fn set_gateway_sender(&mut self, gateway_tx: mpsc::UnboundedSender<GatewayCommandRequest>) {
    self.gateway_tx = Some(gateway_tx);
  }

  /// 启动日志接收器
  pub async fn start(&mut self) -> Result<(), DataError> {
    if !self.config.enabled {
      debug!("任务日志存储已禁用，跳过启动");
      return Ok(());
    }

    // 确保日志目录存在
    tokio::fs::create_dir_all(&self.config.log_dir)
      .await
      .map_err(|e| DataError::server_error(&format!("创建日志目录失败: {}", e)))?;

    let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
    self.shutdown_tx = Some(shutdown_tx);

    let config = self.config.clone();
    let file_writers = self.file_writers.clone();
    let _stats = self.stats.clone();

    // 启动定期清理任务
    tokio::spawn(async move {
      let mut cleanup_interval = interval(config.rotation_check_interval);

      loop {
        tokio::select! {
          _ = cleanup_interval.tick() => {
            if let Err(e) = Self::cleanup_old_logs(&config).await {
              error!("清理旧日志失败: {}", e);
            }

            if let Err(e) = Self::rotate_large_files(&config, &file_writers).await {
              error!("轮转大文件失败: {}", e);
            }
          }
          _ = shutdown_rx.recv() => {
            debug!("收到停止信号，执行最后一次清理");
            let _ = Self::flush_all_writers(&file_writers).await;
            break;
          }
        }
      }
    });

    info!("日志接收器已启动，日志目录: {}", self.config.log_dir);
    Ok(())
  }

  /// 停止日志接收器
  pub async fn stop(&mut self) {
    if let Some(shutdown_tx) = self.shutdown_tx.take() {
      let _ = shutdown_tx.send(());
      debug!("日志接收器停止信号已发送");
    }

    // 刷新所有文件写入器
    if let Err(e) = Self::flush_all_writers(&self.file_writers).await {
      error!("刷新文件写入器失败: {}", e);
    }
  }

  // /// 处理WebSocket日志事件
  // pub async fn handle_log_event(&self, event: &WebSocketEvent) -> Result<(), DataError> {
  //   if !self.config.enabled {
  //     return Ok(());
  //   }

  //   match event.kind {
  //     EventKind::TaskLog => {
  //       if let Some(payload) = &event.payload {
  //         self.process_log_payload(payload).await?
  //       }
  //     }
  //     _ => {
  //       // 忽略非日志事件
  //       return Ok(());
  //     }
  //   }

  //   // 更新统计信息
  //   {
  //     let mut stats = self.stats.write().await;
  //     stats.total_received += 1;
  //     stats.last_update = Some(now_offset());
  //   }

  //   Ok(())
  // }

  /// 处理日志载荷数据
  async fn process_log_payload(&self, payload: &Value) -> Result<(), DataError> {
    // 尝试解析为日志批次
    if let Some(batch_data) = payload.get("batch") {
      let log_batch: LogBatch = serde_json::from_value(batch_data.clone())
        .map_err(|e| DataError::server_error(&format!("解析日志批次失败: {}", e)))?;

      self.write_log_batch(&log_batch).await?
    } else {
      // 尝试解析为单条日志
      let log_message: LogMessage = serde_json::from_value(payload.clone())
        .map_err(|e| DataError::server_error(&format!("解析日志消息失败: {}", e)))?;

      self.write_single_log(&log_message).await?
    }

    Ok(())
  }

  /// 写入日志批次
  pub async fn write_log_batch(&self, batch: &LogBatch) -> Result<(), DataError> {
    debug!("处理日志批次: {}, 包含 {} 条日志", batch.batch_id, batch.messages.len());

    for log_message in &batch.messages {
      self.write_single_log(log_message).await?;
    }

    Ok(())
  }

  /// 写入单条日志
  async fn write_single_log(&self, log: &LogMessage) -> Result<(), DataError> {
    let log_file_path = self.get_log_file_path(&log.task_id.to_string(), &log.task_instance_id.to_string());
    let writer = self.get_or_create_writer(&log_file_path).await?;

    // 格式化日志行
    let log_line = self.format_log_line(log);

    // 写入日志
    {
      let mut writer_guard = writer.lock().await;
      writer_guard
        .write_all(log_line.as_bytes())
        .await
        .map_err(|e| DataError::server_error(&format!("写入日志失败: {}", e)))?;
      writer_guard
        .write_all(b"\n")
        .await
        .map_err(|e| DataError::server_error(&format!("写入换行符失败: {}", e)))?;
      writer_guard
        .flush()
        .await
        .map_err(|e| DataError::server_error(&format!("刷新日志文件失败: {}", e)))?;
    }

    // 通过WebSocket转发日志消息
    self.forward_log_via_websocket(log).await?;

    // 更新统计信息
    {
      let mut stats = self.stats.write().await;
      stats.total_written += 1;
    }

    Ok(())
  }

  /// 通过WebSocket转发日志消息
  async fn forward_log_via_websocket(&self, log_msg: &LogMessage) -> Result<(), DataError> {
    // WebSocket转发功能暂时禁用，等待相关类型定义
    debug!("Log forwarding disabled - task: {}, instance: {}", log_msg.task_id, log_msg.task_instance_id);
    Ok(())
  }

  /// 获取日志文件路径
  fn get_log_file_path(&self, task_id: &str, instance_id: &str) -> PathBuf {
    let date_str = now_offset().date_naive().to_string();

    let filename = format!("{}_{}_{}.log", task_id, instance_id, date_str);
    PathBuf::from(&self.config.log_dir).join(&filename)
  }

  /// 获取或创建文件写入器
  async fn get_or_create_writer(&self, file_path: &PathBuf) -> Result<Arc<Mutex<BufWriter<File>>>, DataError> {
    let path_str = file_path.to_string_lossy().to_string();

    // 先尝试从缓存获取
    {
      let readers = self.file_writers.read().await;
      if let Some(writer) = readers.get(&path_str) {
        return Ok(writer.clone());
      }
    }

    // 创建新的写入器
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(file_path)
      .await
      .map_err(|e| DataError::server_error(&format!("打开日志文件失败: {}", e)))?;

    let writer = Arc::new(Mutex::new(BufWriter::new(file)));

    // 添加到缓存
    {
      let mut writers = self.file_writers.write().await;
      writers.insert(path_str, writer.clone());
    }

    Ok(writer)
  }

  /// 格式化日志行
  fn format_log_line(&self, log: &LogMessage) -> String {
    format!(
      "[{}] [{}] [{}:{}] [{:?}] {}",
      log.timestamp,
      log.level.as_ref().unwrap_or(&"INFO".to_string()),
      log.task_id,
      log.task_instance_id,
      log.log_type,
      log.content
    )
  }

  /// 清理旧日志文件
  async fn cleanup_old_logs(config: &TaskLogConfig) -> Result<(), DataError> {
    // 简化实现：仅记录日志，实际清理逻辑可以后续完善
    debug!("日志清理任务启动，保留天数: {}", config.retention_days);
    Ok(())
  }

  /// 轮转大文件
  async fn rotate_large_files(
    config: &TaskLogConfig,
    file_writers: &Arc<RwLock<HashMap<String, Arc<Mutex<BufWriter<File>>>>>>,
  ) -> Result<(), DataError> {
    let writers_to_rotate = {
      let writers = file_writers.read().await;
      let mut to_rotate = Vec::new();

      for (path, _) in writers.iter() {
        if let Ok(metadata) = tokio::fs::metadata(path).await {
          if metadata.len() > config.max_file_size {
            to_rotate.push(path.clone());
          }
        }
      }

      to_rotate
    };

    for path in writers_to_rotate {
      // 刷新并关闭当前写入器
      {
        let mut writers = file_writers.write().await;
        if let Some(writer) = writers.remove(&path) {
          let mut writer_guard = writer.lock().await;
          let _ = writer_guard.flush().await;
        }
      }

      // 重命名文件
      let timestamp = now_epoch_millis();
      let rotated_path = format!("{}.{}", path, timestamp);

      if let Err(e) = tokio::fs::rename(&path, &rotated_path).await {
        error!("轮转日志文件失败: {} -> {} - {}", path, rotated_path, e);
      } else {
        debug!("已轮转日志文件: {} -> {}", path, rotated_path);
      }
    }

    Ok(())
  }

  /// 刷新所有文件写入器
  async fn flush_all_writers(
    file_writers: &Arc<RwLock<HashMap<String, Arc<Mutex<BufWriter<File>>>>>>,
  ) -> Result<(), DataError> {
    let writers = file_writers.read().await;

    for (path, writer) in writers.iter() {
      let mut writer_guard = writer.lock().await;
      if let Err(e) = writer_guard.flush().await {
        error!("刷新文件写入器失败: {} - {}", path, e);
      }
    }

    Ok(())
  }

  /// 获取统计信息
  pub async fn get_stats(&self) -> LogStats {
    let stats = self.stats.read().await;
    LogStats {
      total_received: stats.total_received,
      total_written: stats.total_written,
      error_count: stats.error_count,
      last_update: stats.last_update,
      websocket_forwarded: stats.websocket_forwarded,
      websocket_failed: stats.websocket_failed,
      avg_processing_latency_ms: stats.avg_processing_latency_ms,
      file_write_failed: stats.file_write_failed,
      consecutive_errors: stats.consecutive_errors,
    }
  }
}

impl Drop for LogReceiver {
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

  #[tokio::test]
  async fn test_log_receiver_creation() {
    let task_config = Arc::new(TaskLogConfig {
      enabled: true,
      log_dir: "/tmp/test_logs".to_string(),
      max_file_size: 1024 * 1024,
      retention_days: 7,
      enable_compression: false,
      rotation_check_interval: Duration::from_secs(60),
      websocket: WebSocketLogConfig {
        enabled: true,
        buffer_size: 1024,
        push_interval: Duration::from_millis(100),
        max_subscribers: 100,
      },
    });

    let websocket_config = Arc::new(task_config.websocket.clone());
    let receiver = LogReceiver::new(task_config, websocket_config);

    assert!(receiver.config.enabled);
    assert_eq!(receiver.config.retention_days, 7);
  }

  #[tokio::test]
  async fn test_log_file_path_generation() {
    let task_config = Arc::new(TaskLogConfig {
      enabled: true,
      log_dir: "/tmp/test_logs".to_string(),
      max_file_size: 1024 * 1024,
      retention_days: 7,
      enable_compression: false,
      rotation_check_interval: Duration::from_secs(60),
      websocket: WebSocketLogConfig {
        enabled: false,
        buffer_size: 1000,
        push_interval: Duration::from_millis(100),
        max_subscribers: 100,
      },
    });

    let websocket_config = Arc::new(task_config.websocket.clone());
    let receiver = LogReceiver::new(task_config, websocket_config);

    let path = receiver.get_log_file_path("task_123", "instance_456");
    let filename = path.file_name().unwrap().to_string_lossy();

    assert!(filename.starts_with("task_123_instance_456_"));
    assert!(filename.ends_with(".log"));
  }
}
