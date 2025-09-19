use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use fusion_common::time::OffsetDateTime;
use fusion_common::time::now_epoch_millis;
use fusion_core::DataError;
use hetuflow_core::protocol::{LogBatch, LogMessage};
use log::{debug, error, info};
use mea::shutdown::ShutdownRecv;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::interval;
use uuid::Uuid;

use crate::gateway::ConnectionManager;
use crate::model::AgentEvent;
use crate::setting::TaskLogConfig;

type FileWriters = HashMap<String, Arc<Mutex<BufWriter<File>>>>;

/// 日志统计信息
#[derive(Debug, Default)]
pub struct LogStats {
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

/// 日志接收器，负责接收Agent转发的日志并存储到文件
pub struct LogReceiver {
  /// 任务日志配置
  config: Arc<TaskLogConfig>,
  /// 文件写入器缓存
  file_writers: Arc<RwLock<FileWriters>>,
  /// 日志统计信息
  stats: Arc<RwLock<LogStats>>,
  log_writer_runner: std::sync::Mutex<Option<LogWriterRunner>>,
  log_clean_runner: std::sync::Mutex<Option<LogCleanRunner>>,
}

impl LogReceiver {
  /// 创建新的日志接收器
  pub fn new(
    config: Arc<TaskLogConfig>,
    shutdown_rx: ShutdownRecv,
    connection_manager: Arc<ConnectionManager>,
  ) -> Result<Self, DataError> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    connection_manager.subscribe_event(event_tx)?;

    let file_writers = Arc::new(RwLock::new(HashMap::new()));
    let stats = Arc::new(RwLock::new(LogStats::default()));

    let log_writer_runner = LogWriterRunner {
      config: config.clone(),
      stats: stats.clone(),
      shutdown_rx: shutdown_rx.clone(),
      event_rx,
      file_writers: file_writers.clone(),
    };

    let log_clean_runner =
      LogCleanRunner { config: config.clone(), shutdown_rx: shutdown_rx.clone(), file_writers: file_writers.clone() };

    Ok(Self {
      config,
      file_writers,
      stats,
      log_writer_runner: std::sync::Mutex::new(Some(log_writer_runner)),
      log_clean_runner: std::sync::Mutex::new(Some(log_clean_runner)),
    })
  }

  /// 启动日志接收器
  pub async fn start(&mut self) -> Result<(), DataError> {
    // 确保日志目录存在
    tokio::fs::create_dir_all(&self.config.log_dir)
      .await
      .map_err(|e| DataError::server_error(format!("创建日志目录失败: {}", e)))?;

    if !self.config.enabled {
      debug!("任务日志存储已禁用，跳过启动");
      return Ok(());
    }

    // 启动处理接收日志任务
    let mut runner = self.log_writer_runner.lock().unwrap().take().unwrap();
    tokio::spawn(async move { runner.run_loop().await });

    // 启动定期清理任务
    let mut runner = self.log_clean_runner.lock().unwrap().take().unwrap();
    tokio::spawn(async move { runner.run_loop().await });

    info!("日志接收器已启动，日志目录: {}", self.config.log_dir);
    Ok(())
  }

  async fn start_clean_task(&self) -> Result<(), DataError> {
    Ok(())
  }

  async fn start_process_loop(&self) -> Result<(), DataError> {
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

struct LogCleanRunner {
  config: Arc<TaskLogConfig>,
  file_writers: Arc<RwLock<FileWriters>>,
  shutdown_rx: ShutdownRecv,
}

impl LogCleanRunner {
  async fn run_loop(&mut self) -> Result<(), DataError> {
    let mut cleanup_interval = interval(self.config.rotation_check_interval);

    loop {
      tokio::select! {
        _ = cleanup_interval.tick() => {
          if let Err(e) = Self::cleanup_old_logs(&self.config).await {
            error!("清理旧日志失败: {}", e);
          }

          if let Err(e) = Self::rotate_large_files(&self.config, &self.file_writers).await {
            error!("轮转大文件失败: {}", e);
          }
        }
        _ = self.shutdown_rx.is_shutdown() => {
          debug!("收到停止信号，执行最后一次清理");
          let _ = Self::flush_all_writers(&self.file_writers).await;
          break;
        }
      }
    }
    Ok(())
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
        if let Ok(metadata) = tokio::fs::metadata(path).await
          && metadata.len() > config.max_file_size
        {
          to_rotate.push(path.clone());
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
}

struct LogWriterRunner {
  config: Arc<TaskLogConfig>,
  stats: Arc<RwLock<LogStats>>,
  shutdown_rx: ShutdownRecv,
  event_rx: mpsc::UnboundedReceiver<AgentEvent>,
  file_writers: Arc<RwLock<FileWriters>>,
}

impl LogWriterRunner {
  pub async fn run_loop(&mut self) -> Result<(), DataError> {
    while let Some(event) = self.event_rx.recv().await
      && !self.shutdown_rx.is_shutdown_now()
    {
      if let AgentEvent::TaskLog { agent_id, payload } = event {
        if let Err(e) = self.process_log_payload(agent_id, payload).await {
          log::warn!("process log payload failed: {:?}", e);
        }
      }
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

  /// 处理日志载荷数据
  async fn process_log_payload(&self, _agent_id: String, payload: Arc<LogMessage>) -> Result<(), DataError> {
    self.write_single_log(&payload).await?;

    Ok(())
  }

  /// 写入单条日志
  async fn write_single_log(&self, log: &LogMessage) -> Result<(), DataError> {
    let log_file_path = self.get_log_file_path(&log.task_instance_id);
    let writer = self.get_or_create_writer(&log_file_path).await?;

    // 格式化日志行
    let log_line = self.format_log_line(log);

    // 写入日志
    {
      let mut writer_guard = writer.lock().await;
      writer_guard
        .write_all(log_line.as_bytes())
        .await
        .map_err(|e| DataError::server_error(format!("写入日志失败: {}", e)))?;
      writer_guard
        .write_all(b"\n")
        .await
        .map_err(|e| DataError::server_error(format!("写入换行符失败: {}", e)))?;
      writer_guard
        .flush()
        .await
        .map_err(|e| DataError::server_error(format!("刷新日志文件失败: {}", e)))?;
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
    debug!("Log forwarding disabled - instance: {}", log_msg.task_instance_id);
    Ok(())
  }

  /// 获取日志文件路径
  fn get_log_file_path(&self, instance_id: &Uuid) -> PathBuf {
    let filename = format!("{}.log", instance_id);
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
    format!("[{}] [{:?}] {}", log.task_instance_id, log.kind, log.content)
  }
}
