use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use hetuflow_core::models::ScheduledTask;
use hetuflow_core::protocol::{
  GatewayCommand, SchedulerEvent, TaskControl, TaskPollRequest, TaskPollResponse, WebSocketEvent,
};
use hetuflow_core::types::{EventKind, TaskInstanceStatus, TaskStatus};
use log::{debug, error, info, warn};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{Instant, interval, sleep};
use ultimate_common::time::OffsetDateTime;
use uuid::Uuid;

/// 任务调度器配置
#[derive(Debug, Clone)]
pub struct TaskSchedulerConfig {
  /// Agent ID
  pub agent_id: Uuid,
  /// Agent 标签
  pub agent_tags: Vec<String>,
  /// 轮询间隔（秒）
  pub poll_interval_seconds: u64,
  /// 最大并发任务数
  pub max_concurrent_tasks: usize,
  /// 任务容量计算权重
  pub capacity_weight: f64,
  /// 负载因子阈值
  pub load_factor_threshold: f64,
  /// 任务超时检查间隔（秒）
  pub timeout_check_interval_seconds: u64,
  /// 默认任务超时时间（秒）
  pub default_task_timeout_seconds: u64,
}

impl Default for TaskSchedulerConfig {
  fn default() -> Self {
    Self {
      agent_id: Uuid::new_v4(),
      agent_tags: vec![],
      poll_interval_seconds: 10,
      max_concurrent_tasks: 5,
      capacity_weight: 1.0,
      load_factor_threshold: 0.8,
      timeout_check_interval_seconds: 30,
      default_task_timeout_seconds: 3600, // 1小时
    }
  }
}

/// 任务调度状态
#[derive(Debug, Clone)]
pub struct TaskSchedulingState {
  /// 活跃任务数
  pub active_tasks: usize,
  /// 当前负载因子
  pub load_factor: f64,
  /// 可用容量
  pub available_capacity: usize,
  /// 最后轮询时间
  pub last_poll_time: Option<Instant>,
  /// 轮询计数
  pub poll_count: u64,
}

impl Default for TaskSchedulingState {
  fn default() -> Self {
    Self { active_tasks: 0, load_factor: 0.0, available_capacity: 0, last_poll_time: None, poll_count: 0 }
  }
}

/// 任务调度器
/// 负责 Agent Poll 机制、任务调度和容量计算
#[derive(Debug)]
pub struct TaskScheduler {
  /// 配置
  config: TaskSchedulerConfig,
  /// 调度状态
  state: Arc<RwLock<TaskSchedulingState>>,
  /// 待调度任务队列
  pending_tasks: Arc<RwLock<Vec<ScheduledTask>>>,
  /// 活跃任务映射 (task_instance_id -> ScheduledTask)
  active_tasks: Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
  /// 事件发送器（发送给 ConnectionManager）
  event_sender: mpsc::UnboundedSender<WebSocketEvent>,
  /// 事件接收器
  event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<WebSocketEvent>>>>,
  /// 命令发送器（发送给 TaskExecutor）
  command_sender: mpsc::UnboundedSender<GatewayCommand>,
  /// 命令接收器
  command_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<GatewayCommand>>>>,
}

impl TaskScheduler {
  /// 创建新的任务调度器
  pub fn new(config: TaskSchedulerConfig) -> Self {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let (command_sender, command_receiver) = mpsc::unbounded_channel();

    Self {
      config,
      state: Arc::new(RwLock::new(TaskSchedulingState::default())),
      pending_tasks: Arc::new(RwLock::new(Vec::new())),
      active_tasks: Arc::new(RwLock::new(HashMap::default())),
      event_sender,
      event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
      command_sender,
      command_receiver: Arc::new(RwLock::new(Some(command_receiver))),
    }
  }

  /// 获取事件发送器
  pub fn get_event_sender(&self) -> mpsc::UnboundedSender<SchedulerEvent> {
    self.event_sender.clone()
  }

  /// 获取命令接收器
  pub async fn take_command_receiver(&self) -> Option<mpsc::UnboundedReceiver<GatewayCommand>> {
    let mut receiver_guard = self.command_receiver.write().await;
    receiver_guard.take()
  }

  /// 获取调度状态
  pub async fn get_state(&self) -> TaskSchedulingState {
    let state = self.state.read().await;
    state.clone()
  }

  /// 获取活跃任务数
  pub async fn get_active_task_count(&self) -> usize {
    let active_tasks = self.active_tasks.read().await;
    active_tasks.len()
  }

  /// 获取负载因子
  pub async fn get_load_factor(&self) -> f64 {
    let state = self.state.read().await;
    state.load_factor
  }

  /// 启动任务调度器
  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting TaskScheduler");

    // 启动轮询循环
    let poll_state = Arc::clone(&self.state);
    let poll_config = self.config.clone();
    let poll_event_sender = self.event_sender.clone();
    let poll_pending_tasks = Arc::clone(&self.pending_tasks);
    let poll_active_tasks = Arc::clone(&self.active_tasks);

    tokio::spawn(async move {
      Self::polling_loop(poll_config, poll_state, poll_event_sender, poll_pending_tasks, poll_active_tasks).await;
    });

    // 启动事件处理循环
    let event_receiver = {
      let mut receiver_guard = self.event_receiver.write().await;
      receiver_guard.take()
    };

    if let Some(receiver) = event_receiver {
      let event_state = Arc::clone(&self.state);
      let event_pending_tasks = Arc::clone(&self.pending_tasks);
      let event_active_tasks = Arc::clone(&self.active_tasks);
      let event_command_sender = self.command_sender.clone();

      tokio::spawn(async move {
        Self::event_processing_loop(
          receiver,
          event_state,
          event_pending_tasks,
          event_active_tasks,
          event_command_sender,
        )
        .await;
      });
    }

    // 启动超时检查循环
    let timeout_state = Arc::clone(&self.state);
    let timeout_config = self.config.clone();
    let timeout_active_tasks = Arc::clone(&self.active_tasks);
    let timeout_command_sender = self.command_sender.clone();

    tokio::spawn(async move {
      Self::timeout_check_loop(timeout_config, timeout_state, timeout_active_tasks, timeout_command_sender).await;
    });

    info!("TaskScheduler started successfully");
    Ok(())
  }

  /// 添加待调度任务
  pub async fn add_pending_task(&self, task: ScheduledTask) {
    let mut pending_tasks = self.pending_tasks.write().await;
    pending_tasks.push(task);
    debug!("Added pending task, total pending: {}", pending_tasks.len());
  }

  /// 标记任务为活跃状态
  pub async fn mark_task_active(&self, task_instance_id: Uuid, task: ScheduledTask) {
    let mut active_tasks = self.active_tasks.write().await;
    active_tasks.insert(task_instance_id, task);

    // 更新状态
    let mut state = self.state.write().await;
    state.active_tasks = active_tasks.len();
    state.load_factor = self.calculate_load_factor(active_tasks.len());
    state.available_capacity = self.calculate_available_capacity(active_tasks.len());

    debug!("Marked task {} as active, total active: {}", task_instance_id, active_tasks.len());
  }

  /// 移除活跃任务
  pub async fn remove_active_task(&self, task_instance_id: &Uuid) {
    let mut active_tasks = self.active_tasks.write().await;
    if active_tasks.remove(task_instance_id).is_some() {
      // 更新状态
      let mut state = self.state.write().await;
      state.active_tasks = active_tasks.len();
      state.load_factor = self.calculate_load_factor(active_tasks.len());
      state.available_capacity = self.calculate_available_capacity(active_tasks.len());

      debug!("Removed active task {}, total active: {}", task_instance_id, active_tasks.len());
    }
  }

  /// 计算负载因子
  fn calculate_load_factor(&self, active_tasks: usize) -> f64 {
    if self.config.max_concurrent_tasks == 0 {
      return 0.0;
    }

    let ratio = active_tasks as f64 / self.config.max_concurrent_tasks as f64;
    (ratio * self.config.capacity_weight).min(1.0)
  }

  /// 计算可用容量
  fn calculate_available_capacity(&self, active_tasks: usize) -> usize {
    if active_tasks >= self.config.max_concurrent_tasks { 0 } else { self.config.max_concurrent_tasks - active_tasks }
  }

  /// 轮询循环
  async fn polling_loop(
    config: TaskSchedulerConfig,
    state: Arc<RwLock<TaskSchedulingState>>,
    event_sender: mpsc::UnboundedSender<WebSocketEvent>,
    pending_tasks: Arc<RwLock<Vec<ScheduledTask>>>,
    active_tasks: Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
  ) {
    info!("TaskScheduler polling loop started");

    let mut poll_interval = interval(Duration::from_secs(config.poll_interval_seconds));

    loop {
      poll_interval.tick().await;

      // 更新轮询状态
      let (current_capacity, current_load_factor) = {
        let mut state_guard = state.write().await;
        let active_count = {
          let active_tasks_guard = active_tasks.read().await;
          active_tasks_guard.len()
        };

        state_guard.active_tasks = active_count;
        state_guard.load_factor = if config.max_concurrent_tasks == 0 {
          0.0
        } else {
          (active_count as f64 / config.max_concurrent_tasks as f64 * config.capacity_weight).min(1.0)
        };
        state_guard.available_capacity =
          if active_count >= config.max_concurrent_tasks { 0 } else { config.max_concurrent_tasks - active_count };
        state_guard.last_poll_time = Some(Instant::now());
        state_guard.poll_count += 1;

        (state_guard.available_capacity, state_guard.load_factor)
      };

      // 检查是否需要轮询新任务
      if current_capacity > 0 && current_load_factor < config.load_factor_threshold {
        debug!(
          "Polling for new tasks, available capacity: {}, load factor: {:.2}",
          current_capacity, current_load_factor
        );

        // 发送轮询请求
        let poll_request = TaskPollRequest {
          agent_id: config.agent_id,
          max_tasks: current_capacity as u32,
          available_capacity: current_capacity as u32,
          tags: config.agent_tags.clone(),
        };

        if let Err(e) = event_sender.send(SchedulerEvent::TaskPollRequest(poll_request)) {
          error!("Failed to send poll request: {}", e);
        }
      } else {
        debug!("Skipping poll - capacity: {}, load factor: {:.2}", current_capacity, current_load_factor);
      }

      // 处理待调度任务
      Self::process_pending_tasks(&config, &state, &pending_tasks, &active_tasks, &event_sender).await;
    }
  }

  /// 处理待调度任务
  async fn process_pending_tasks(
    config: &TaskSchedulerConfig,
    state: &Arc<RwLock<TaskSchedulingState>>,
    pending_tasks: &Arc<RwLock<Vec<ScheduledTask>>>,
    active_tasks: &Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    event_sender: &mpsc::UnboundedSender<SchedulerEvent>,
  ) {
    let mut tasks_to_schedule = Vec::new();

    // 获取可调度的任务
    {
      let mut pending_guard = pending_tasks.write().await;
      let state_guard = state.read().await;

      let available_slots = state_guard.available_capacity;
      if available_slots > 0 {
        // 按优先级排序并选择要调度的任务
        pending_guard.sort_by(|a, b| {
          b.task.priority.cmp(&a.task.priority).then_with(|| a.task.created_at.cmp(&b.task.created_at))
        });

        let tasks_to_take = available_slots.min(pending_guard.len());
        tasks_to_schedule = pending_guard.drain(0..tasks_to_take).collect();
      }
    }

    // 调度选中的任务
    for task in tasks_to_schedule {
      debug!("Scheduling task: {} (job: {})", task.task.id, task.task.job_id);

      // 发送任务调度事件
      if let Err(e) = event_sender.send(SchedulerEvent::TaskScheduled(task)) {
        error!("Failed to send task scheduled event: {}", e);
      }
    }
  }

  /// 事件处理循环
  async fn event_processing_loop(
    mut receiver: mpsc::UnboundedReceiver<WebSocketEvent>,
    state: Arc<RwLock<TaskSchedulingState>>,
    pending_tasks: Arc<RwLock<Vec<ScheduledTask>>>,
    active_tasks: Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    command_sender: mpsc::UnboundedSender<GatewayCommand>,
  ) {
    info!("TaskScheduler event processing loop started");

    while let Some(event) = receiver.recv().await {
      match event.kind {
        EventKind::TaskPollResponse => {
          Self::handle_poll_response(response, &pending_tasks).await;
        }
        EventKind::TaskScheduled => {
          Self::handle_task_scheduled(task, &command_sender).await;
        }
        EventKind::TaskCompleted => {
          Self::handle_task_completed(task, &active_tasks, &state).await;
        }
        EventKind::TaskFailed => {
          Self::handle_task_failed(task, &active_tasks, &state).await;
        }
        _ => {
          debug!("Received unhandled event: {:?}", event);
        }
      }
    }

    info!("TaskScheduler event processing loop ended");
  }

  /// 处理轮询响应
  async fn handle_poll_response(response: TaskPollResponse, pending_tasks: &Arc<RwLock<Vec<ScheduledTask>>>) {
    if !response.tasks.is_empty() {
      info!("Received {} tasks from poll response", response.tasks.len());

      let mut pending_guard = pending_tasks.write().await;
      for task in response.tasks {
        pending_guard.push(task);
      }

      debug!("Added tasks to pending queue, total pending: {}", pending_guard.len());
    }
  }

  /// 处理任务调度
  async fn handle_task_scheduled(task: ScheduledTask, command_sender: &mpsc::UnboundedSender<GatewayCommand>) {
    debug!("Handling scheduled task: {}", task.task.id);

    // 发送任务执行命令
    let command = GatewayCommand::ExecuteTask {
      task_id: task.task.id,
      task_instance_id: Uuid::new_v4(), // TODO: 从任务实例获取
      payload: task.task.payload.clone(),
    };

    if let Err(e) = command_sender.send(command) {
      error!("Failed to send execute task command: {}", e);
    }
  }

  /// 处理任务完成
  async fn handle_task_completed(
    task_instance_id: Uuid,
    result: String,
    active_tasks: &Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    state: &Arc<RwLock<TaskSchedulingState>>,
  ) {
    info!("Task {} completed with result: {}", task_instance_id, result);

    // 从活跃任务中移除
    let mut active_guard = active_tasks.write().await;
    if active_guard.remove(&task_instance_id).is_some() {
      // 更新状态
      let mut state_guard = state.write().await;
      state_guard.active_tasks = active_guard.len();

      debug!("Removed completed task from active tasks, remaining: {}", active_guard.len());
    }
  }

  /// 处理任务失败
  async fn handle_task_failed(
    task_instance_id: Uuid,
    error: String,
    active_tasks: &Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    state: &Arc<RwLock<TaskSchedulingState>>,
  ) {
    warn!("Task {} failed with error: {}", task_instance_id, error);

    // 从活跃任务中移除
    let mut active_guard = active_tasks.write().await;
    if let Some(task) = active_guard.remove(&task_instance_id) {
      // 更新状态
      let mut state_guard = state.write().await;
      state_guard.active_tasks = active_guard.len();

      debug!("Removed failed task from active tasks, remaining: {}", active_guard.len());

      // TODO: 实现重试逻辑
      // 根据任务的重试策略决定是否重新调度
    }
  }

  /// 超时检查循环
  async fn timeout_check_loop(
    config: TaskSchedulerConfig,
    state: Arc<RwLock<TaskSchedulingState>>,
    active_tasks: Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    command_sender: mpsc::UnboundedSender<GatewayCommand>,
  ) {
    info!("TaskScheduler timeout check loop started");

    let mut timeout_interval = interval(Duration::from_secs(config.timeout_check_interval_seconds));

    loop {
      timeout_interval.tick().await;

      let now = now_offset();
      let mut timed_out_tasks = Vec::new();

      // 检查超时任务
      {
        let active_guard = active_tasks.read().await;
        for (task_instance_id, task) in active_guard.iter() {
          let timeout_duration =
            Duration::from_secs(task.task.timeout_seconds.unwrap_or(config.default_task_timeout_seconds));

          if let Some(started_at) = task.task.started_at {
            let elapsed = now - started_at;
            if elapsed.whole_seconds() as u64 > timeout_duration.as_secs() {
              timed_out_tasks.push(*task_instance_id);
            }
          }
        }
      }

      // 处理超时任务
      for task_instance_id in timed_out_tasks {
        warn!("Task {} timed out, sending kill command", task_instance_id);

        let command = GatewayCommand::KillTask { task_instance_id };
        if let Err(e) = command_sender.send(command) {
          error!("Failed to send kill task command: {}", e);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hetuflow_core::models::{ScheduleEntity, TaskEntity};
  use hetuflow_core::types::{ScheduleType, TaskPriority};

  #[tokio::test]
  async fn test_task_scheduler_creation() {
    let config = TaskSchedulerConfig::default();
    let scheduler = TaskScheduler::new(config);

    let state = scheduler.get_state().await;
    assert_eq!(state.active_tasks, 0);
    assert_eq!(state.load_factor, 0.0);
  }

  #[tokio::test]
  async fn test_load_factor_calculation() {
    let config = TaskSchedulerConfig { max_concurrent_tasks: 10, capacity_weight: 1.0, ..Default::default() };
    let scheduler = TaskScheduler::new(config);

    assert_eq!(scheduler.calculate_load_factor(0), 0.0);
    assert_eq!(scheduler.calculate_load_factor(5), 0.5);
    assert_eq!(scheduler.calculate_load_factor(10), 1.0);
    assert_eq!(scheduler.calculate_load_factor(15), 1.0); // 不应超过 1.0
  }

  #[tokio::test]
  async fn test_capacity_calculation() {
    let config = TaskSchedulerConfig { max_concurrent_tasks: 10, ..Default::default() };
    let scheduler = TaskScheduler::new(config);

    assert_eq!(scheduler.calculate_available_capacity(0), 10);
    assert_eq!(scheduler.calculate_available_capacity(5), 5);
    assert_eq!(scheduler.calculate_available_capacity(10), 0);
    assert_eq!(scheduler.calculate_available_capacity(15), 0);
  }

  #[tokio::test]
  async fn test_add_pending_task() {
    let config = TaskSchedulerConfig::default();
    let scheduler = TaskScheduler::new(config);

    let task = create_test_scheduled_task();
    scheduler.add_pending_task(task).await;

    let pending_tasks = scheduler.pending_tasks.read().await;
    assert_eq!(pending_tasks.len(), 1);
  }

  #[tokio::test]
  async fn test_mark_task_active() {
    let config = TaskSchedulerConfig::default();
    let scheduler = TaskScheduler::new(config);

    let task = create_test_scheduled_task();
    let task_instance_id = Uuid::new_v4();

    scheduler.mark_task_active(task_instance_id, task).await;

    let active_count = scheduler.get_active_task_count().await;
    assert_eq!(active_count, 1);

    let load_factor = scheduler.get_load_factor().await;
    assert!(load_factor > 0.0);
  }

  #[tokio::test]
  async fn test_remove_active_task() {
    let config = TaskSchedulerConfig::default();
    let scheduler = TaskScheduler::new(config);

    let task = create_test_scheduled_task();
    let task_instance_id = Uuid::new_v4();

    scheduler.mark_task_active(task_instance_id, task).await;
    assert_eq!(scheduler.get_active_task_count().await, 1);

    scheduler.remove_active_task(&task_instance_id).await;
    assert_eq!(scheduler.get_active_task_count().await, 0);
  }

  fn create_test_scheduled_task() -> ScheduledTask {
    let task = TaskEntity {
      id: Uuid::new_v4(),
      job_id: Uuid::new_v4(),
      name: "test_task".to_string(),
      description: Some("Test task".to_string()),
      payload: serde_json::json!({"command": "echo hello"}),
      priority: TaskPriority::Medium,
      tags: vec!["test".to_string()],
      timeout_seconds: Some(300),
      retry_count: 0,
      max_retries: 3,
      status: TaskStatus::Pending,
      created_at: now_offset(),
      updated_at: now_offset(),
      started_at: None,
      completed_at: None,
    };

    let schedule = ScheduleEntity {
      id: Uuid::new_v4(),
      job_id: task.job_id,
      name: "test_schedule".to_string(),
      description: Some("Test schedule".to_string()),
      schedule_type: ScheduleType::Manual,
      cron_expression: None,
      timezone: "UTC".to_string(),
      is_active: true,
      created_at: now_offset(),
      updated_at: now_offset(),
      last_run_at: None,
      next_run_at: None,
    };

    ScheduledTask::new(task, schedule)
  }

  #[test]
  fn test_task_scheduler_config_default() {
    let config = TaskSchedulerConfig::default();

    assert_eq!(config.poll_interval_seconds, 10);
    assert_eq!(config.max_concurrent_tasks, 5);
    assert_eq!(config.capacity_weight, 1.0);
    assert_eq!(config.load_factor_threshold, 0.8);
    assert_eq!(config.timeout_check_interval_seconds, 30);
    assert_eq!(config.default_task_timeout_seconds, 3600);
  }
}
