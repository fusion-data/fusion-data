use std::{
  collections::HashMap,
  process::Stdio,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use fusion_common::time::now_epoch_millis;
use fusion_core::DataError;
use log::{debug, error, info, warn};
use tokio::{process::Command, sync::RwLock};
use uuid::Uuid;

use hetuflow_core::{
  models::TaskMetrics,
  protocol::{
    BackoffStrategy, ProcessInfo, ScheduledTask, TaskExecutionError, TaskExecutionErrorType, TaskExecutionResult,
    TaskInstanceUpdated, WebSocketEvent,
  },
  types::{EventKind, TaskInstanceStatus},
};

use crate::{
  service::{ConnectionManager, ProcessManager},
  setting::HetuflowAgentSetting,
};

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
  /// 基础延迟时间（毫秒）
  pub base_delay_ms: u64,
  /// 最大延迟时间（毫秒）
  pub max_delay_ms: u64,
  /// 退避策略
  pub backoff_strategy: BackoffStrategy,
}

impl Default for RetryConfig {
  fn default() -> Self {
    Self { base_delay_ms: 1000, max_delay_ms: 60000, backoff_strategy: BackoffStrategy::Exponential }
  }
}

/// 任务执行状态
#[derive(Debug, Clone)]
pub struct TaskExecution {
  /// 任务实例ID
  pub instance_id: Uuid,
  /// 调度任务
  pub task: ScheduledTask,
  /// 进程ID
  pub process_id: Option<Uuid>,
  /// 进程信息
  pub process_info: Option<ProcessInfo>,
  /// 开始时间
  pub started_at: i64,
  /// 重试次数
  pub retry_count: i32,
  /// 状态
  pub status: TaskInstanceStatus,
  /// 取消标志
  pub cancelled: Arc<AtomicBool>,
}

/// 任务执行器。负责执行具体的任务，包括进程管理、重试机制和状态上报
pub struct TaskExecutor {
  /// Agent ID
  setting: Arc<HetuflowAgentSetting>,
  /// 进程管理器
  process_manager: Arc<ProcessManager>,
  /// 活跃任务执行
  active_executions: Arc<RwLock<HashMap<Uuid, TaskExecution>>>,
  /// 连接管理器
  connection_manager: Arc<ConnectionManager>,
  /// 重试配置
  retry_config: RetryConfig,
  scheduled_task_rx: Arc<kanal::AsyncReceiver<ScheduledTask>>,
}

impl TaskExecutor {
  /// 创建新的任务执行器
  pub fn new(
    setting: Arc<HetuflowAgentSetting>,
    process_manager: Arc<ProcessManager>,
    connection_manager: Arc<ConnectionManager>,
    retry_config: Option<RetryConfig>,
    scheduled_task_rx: kanal::AsyncReceiver<ScheduledTask>,
  ) -> Self {
    Self {
      setting,
      process_manager,
      active_executions: Arc::new(RwLock::new(HashMap::new())),
      connection_manager,
      retry_config: retry_config.unwrap_or_default(),
      scheduled_task_rx: Arc::new(scheduled_task_rx),
    }
  }

  /// 启动任务执行器
  pub async fn run_loop(&self) -> Result<(), DataError> {
    info!("Starting TaskExecutor for agent {}", self.setting.agent_id);

    loop {
      match self.scheduled_task_rx.recv().await {
        Ok(task) => {
          if let Err(e) = self.execute_task(task).await {
            error!("Failed to execute task: {:?}", e);
          }
        }
        Err(e) => {
          info!("Task schedule channel closed: {}", e);
          break;
        }
      }
    }

    self.stop().await
  }

  /// 停止任务执行器
  async fn stop(&self) -> Result<(), DataError> {
    info!("Stopping TaskExecutor for agent {}", self.setting.agent_id);

    // 取消所有活跃任务
    let executions = self.active_executions.read().await;
    for execution in executions.values() {
      execution.cancelled.store(true, Ordering::SeqCst);
      if let Some(process_id) = execution.process_id
        && let Err(e) = self.process_manager.kill_process(process_id).await
      {
        warn!("Failed to kill process {}: {}", process_id, e);
      }
    }
    drop(executions);

    // 等待所有任务完成
    let mut attempts = 0;
    while !self.active_executions.read().await.is_empty() && attempts < 30 {
      tokio::time::sleep(Duration::from_millis(100)).await;
      attempts += 1;
    }

    Ok(())
  }

  /// 执行任务
  pub async fn execute_task(&self, task: ScheduledTask) -> Result<TaskExecutionResult, TaskExecutionError> {
    let instance_id = Uuid::now_v7();
    let execution = TaskExecution {
      instance_id,
      task: task.clone(),
      process_id: None,
      process_info: None,
      started_at: now_epoch_millis(),
      retry_count: 0,
      status: TaskInstanceStatus::Running,
      cancelled: Arc::new(AtomicBool::new(false)),
    };

    // 添加到活跃执行列表
    self.active_executions.write().await.insert(instance_id, execution.clone());

    // 执行任务（带重试）
    let result = self.execute_with_retry(execution).await;

    // 从活跃执行列表中移除
    self.active_executions.write().await.remove(&instance_id);

    result
  }

  /// 取消任务
  pub async fn cancel_task(&self, instance_id: Uuid) -> Result<(), DataError> {
    let executions = self.active_executions.read().await;
    if let Some(execution) = executions.get(&instance_id) {
      execution.cancelled.store(true, Ordering::SeqCst);
      if let Some(process_id) = execution.process_id {
        self.process_manager.kill_process(process_id).await?;
      }
    }
    Ok(())
  }

  /// 获取活跃任务列表
  pub async fn get_active_tasks(&self) -> Vec<TaskExecution> {
    self.active_executions.read().await.values().cloned().collect()
  }

  /// 带重试的任务执行
  async fn execute_with_retry(&self, mut execution: TaskExecution) -> Result<TaskExecutionResult, TaskExecutionError> {
    let max_retries = execution.task.task.max_retries;

    loop {
      // 检查是否被取消
      if execution.cancelled.load(Ordering::SeqCst) {
        return Err(TaskExecutionError {
          task_id: execution.task.task.id,
          instance_id: Some(execution.instance_id),
          error_type: TaskExecutionErrorType::Cancelled,
          message: "Task was cancelled".to_string(),
          retry_count: execution.retry_count,
          max_retries,
          timestamp: now_epoch_millis(),
        });
      }

      // 执行任务
      match self.execute_single_attempt(&mut execution).await {
        Ok(result) => {
          // 发送成功事件
          let ret = result.clone();
          let _ = self.connection_manager.send_event(WebSocketEvent::new(
            EventKind::TaskChangedEvent,
            TaskInstanceUpdated {
              task_instance_id: execution.instance_id,
              task_id: execution.task.task.id,
              agent_id: self.setting.agent_id,
              timestamp: now_epoch_millis(),
              output: result.output,
              error_message: result.error_message,
              exit_code: result.exit_code,
              metrics: result.metrics,
              progress: Some(1.0),
              status: TaskInstanceStatus::Succeeded,
            },
          ));
          return Ok(ret);
        }
        Err(error) => {
          execution.retry_count += 1;

          // 检查是否还能重试
          if execution.retry_count >= max_retries {
            // 发送失败事件
            let _ = self.connection_manager.send_event(WebSocketEvent::new(
              EventKind::TaskChangedEvent,
              TaskInstanceUpdated {
                task_instance_id: execution.instance_id,
                task_id: execution.task.task.id,
                agent_id: self.setting.agent_id,
                timestamp: now_epoch_millis(),
                output: None,
                error_message: Some(error.message.clone()),
                exit_code: None,
                metrics: None,
                progress: None,
                status: TaskInstanceStatus::Failed,
              },
            ));
            return Err(error);
          }

          // 计算重试延迟
          let delay = self.calculate_retry_delay(execution.retry_count);
          warn!(
            "Task {} failed (attempt {}/{}), retrying in {}ms: {}",
            execution.task.task.id, execution.retry_count, max_retries, delay, error.message
          );

          // 等待重试延迟
          tokio::time::sleep(Duration::from_millis(delay)).await;
        }
      }
    }
  }

  /// 执行单次任务尝试
  async fn execute_single_attempt(
    &self,
    execution: &mut TaskExecution,
  ) -> Result<TaskExecutionResult, TaskExecutionError> {
    let start_time = std::time::Instant::now();

    // 获取任务配置
    let job_config = execution.task.task.job_config.as_ref().ok_or_else(|| TaskExecutionError {
      task_id: execution.task.task.id,
      instance_id: Some(execution.instance_id),
      error_type: TaskExecutionErrorType::ConfigurationError,
      message: "Missing job configuration".to_string(),
      retry_count: execution.retry_count,
      max_retries: execution.task.task.max_retries,
      timestamp: now_epoch_millis(),
    })?;

    // 构建命令
    // TODO:
    let mut cmd = Command::new("python");

    // 设置工作目录
    if let Some(working_dir) = &job_config.working_directory {
      cmd.current_dir(working_dir);
    }

    // 设置环境变量
    if let Some(value) = execution.task.task.environment.clone() {
      // TODO: 更好的错误处理
      let envs: HashMap<String, String> = serde_json::from_value(value).unwrap();
      for (key, value) in envs {
        cmd.env(key, value);
      }
    }

    // 设置标准输入输出
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

    // 启动进程
    let mut child = cmd.spawn().map_err(|e| TaskExecutionError {
      task_id: execution.task.task.id,
      instance_id: Some(execution.instance_id),
      error_type: TaskExecutionErrorType::ProcessStartFailed,
      message: format!("Failed to spawn process: {}", e),
      retry_count: execution.retry_count,
      max_retries: execution.task.task.max_retries,
      timestamp: now_epoch_millis(),
    })?;

    let pid = child.id().unwrap_or(0);
    debug!("Started process {} for task {}", pid, execution.task.task.id);

    // 等待进程完成
    let output = child.wait_with_output().await.map_err(|e| TaskExecutionError {
      task_id: execution.task.task.id,
      instance_id: Some(execution.instance_id),
      error_type: TaskExecutionErrorType::ProcessTimeout, // TODO: 是否有更合适的错误类型？
      message: format!("Process execution failed: {}", e),
      retry_count: execution.retry_count,
      max_retries: execution.task.task.max_retries,
      timestamp: now_epoch_millis(),
    })?;

    let duration = start_time.elapsed();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    debug!("Process {} completed with exit code {} in {}ms", pid, exit_code, duration.as_millis());

    // 更新执行状态
    execution.status = if success { TaskInstanceStatus::Succeeded } else { TaskInstanceStatus::Failed };

    // 创建任务指标
    let metrics = TaskMetrics {
      start_time: execution.started_at,
      end_time: Some(now_epoch_millis()),
      cpu_time: 0.0,
      memory_peak: 0,
      disk_read: 0,
      disk_write: 0,
      network_in: 0,
      network_out: 0,
    };

    // 发送任务状态更新
    let task_update = TaskInstanceUpdated {
      task_instance_id: execution.instance_id,
      task_id: execution.task.task.id,
      agent_id: self.setting.agent_id,
      status: execution.status.clone(),
      timestamp: now_epoch_millis(),
      output: Some(stdout.clone()),
      error_message: if success { None } else { Some(stderr.clone()) },
      exit_code: Some(exit_code),
      metrics: Some(metrics.clone()),
      progress: None,
    };

    // TODO: 这里应该发送 task_update 到服务器，但现在先跳过
    debug!("Task update: {:?}", task_update);

    if success {
      Ok(TaskExecutionResult {
        task_id: execution.task.task.id,
        instance_id: execution.instance_id,
        success: true,
        exit_code: Some(exit_code),
        output: Some(stdout),
        error_message: None,
        duration_ms: duration.as_millis() as u64,
        metrics: Some(metrics),
      })
    } else {
      Err(TaskExecutionError {
        task_id: execution.task.task.id,
        instance_id: Some(execution.instance_id),
        error_type: TaskExecutionErrorType::ProcessStartFailed, // TODO: 是否有更合适的错误类型？
        message: format!("Process failed with exit code {}: {}", exit_code, stderr),
        retry_count: execution.retry_count,
        max_retries: execution.task.task.max_retries,
        timestamp: now_epoch_millis(),
      })
    }
  }

  /// 计算重试延迟
  fn calculate_retry_delay(&self, retry_count: i32) -> u64 {
    match self.retry_config.backoff_strategy {
      BackoffStrategy::Fixed => self.retry_config.base_delay_ms,
      BackoffStrategy::Linear => {
        let delay = self.retry_config.base_delay_ms * retry_count as u64;
        delay.min(self.retry_config.max_delay_ms)
      }
      BackoffStrategy::Exponential => {
        let delay = self.retry_config.base_delay_ms * (2_u64.pow(retry_count as u32 - 1));
        delay.min(self.retry_config.max_delay_ms)
      }
      BackoffStrategy::ExponentialWithJitter => {
        let base_delay = self.retry_config.base_delay_ms * (2_u64.pow(retry_count as u32 - 1));
        let jitter = (base_delay as f64 * 0.1 * 0.5) as u64; // 简化的抖动
        let delay = base_delay + jitter;
        delay.min(self.retry_config.max_delay_ms)
      }
    }
  }
}
