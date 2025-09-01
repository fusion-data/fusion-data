use std::{collections::VecDeque, sync::Arc, time::Duration};

use ahash::HashMap;
use hetumind_core::workflow::{
  ExecutionConfig, ExecutionData, ExecutionId, NodeExecutionError, NodeName, WorkflowExecutionError,
};
use tokio::sync::{RwLock, mpsc};
use fusion_common::time::now;

use super::{ExecutionTask, RunningTask, SchedulerCommand, WaitingTask};

/// 最大并发任务数
// TODO 从配置中获取
const MAX_CONCURRENT_TASKS: usize = 100;

/// 项目中的 TaskScheduler（进程内调度器）
///这是一个进程内的任务调度器，用于单个工作流执行内部的节点调度：
pub struct TaskScheduler {
  /// 待执行任务队列
  task_queue: Arc<RwLock<VecDeque<ExecutionTask>>>,
  /// 等待依赖的任务
  waiting_tasks: Arc<RwLock<HashMap<NodeName, WaitingTask>>>,
  /// 正在执行的任务
  running_tasks: Arc<RwLock<HashMap<NodeName, RunningTask>>>,
  /// 任务调度器
  scheduler_handle: Option<tokio::task::JoinHandle<()>>,
  /// 控制通道
  control_tx: mpsc::UnboundedSender<SchedulerCommand>,
  control_rx: Arc<RwLock<mpsc::UnboundedReceiver<SchedulerCommand>>>,
}

impl TaskScheduler {
  pub fn new(config: ExecutionConfig) -> Self {
    let (control_tx, control_rx) = mpsc::unbounded_channel();

    Self {
      task_queue: Arc::new(RwLock::new(VecDeque::new())),
      waiting_tasks: Arc::new(RwLock::new(HashMap::default())),
      running_tasks: Arc::new(RwLock::new(HashMap::default())),
      scheduler_handle: None,
      control_tx,
      control_rx: Arc::new(RwLock::new(control_rx)),
    }
  }

  pub async fn start(&mut self) {
    let task_queue = Arc::clone(&self.task_queue);
    let waiting_tasks = Arc::clone(&self.waiting_tasks);
    let running_tasks = Arc::clone(&self.running_tasks);
    let control_rx = Arc::clone(&self.control_rx);

    self.scheduler_handle = Some(tokio::spawn(async move {
      Self::scheduler_loop(task_queue, waiting_tasks, running_tasks, control_rx).await;
    }));
  }

  async fn scheduler_loop(
    task_queue: Arc<RwLock<VecDeque<ExecutionTask>>>,
    waiting_tasks: Arc<RwLock<HashMap<NodeName, WaitingTask>>>,
    running_tasks: Arc<RwLock<HashMap<NodeName, RunningTask>>>,
    control_rx: Arc<RwLock<mpsc::UnboundedReceiver<SchedulerCommand>>>,
  ) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

    loop {
      tokio::select! {
          _ = interval.tick() => {
              // 检查超时任务
              Self::check_task_timeouts(&running_tasks).await;

              // 调度新任务
              Self::schedule_ready_tasks(&task_queue, &waiting_tasks, &running_tasks).await;
          }

          command = Self::receive_command(&control_rx) => {
              match command {
                  Some(SchedulerCommand::Shutdown) => break,
                  Some(cmd) => {
                      Self::handle_command(cmd, &task_queue, &waiting_tasks, &running_tasks).await;
                  }
                  None => break,
              }
          }
      }
    }
  }

  async fn schedule_ready_tasks(
    task_queue: &Arc<RwLock<VecDeque<ExecutionTask>>>,
    waiting_tasks: &Arc<RwLock<HashMap<NodeName, WaitingTask>>>,
    running_tasks: &Arc<RwLock<HashMap<NodeName, RunningTask>>>,
  ) {
    // 从队列中取出就绪的任务并执行
    let mut queue = task_queue.write().await;
    let mut waiting = waiting_tasks.write().await;
    let mut running = running_tasks.write().await;

    // 检查等待中的任务是否就绪
    let mut ready_tasks = Vec::new();
    waiting.retain(|node_name, waiting_task| {
      if waiting_task.waiting_for.is_empty() {
        ready_tasks.push((node_name.clone(), waiting_task.task.clone()));
        false
      } else {
        true
      }
    });

    // 将就绪的任务加入执行队列
    for (_, task) in ready_tasks {
      queue.push_back(task);
    }

    // 按优先级排序
    queue.make_contiguous().sort_by_key(|task| std::cmp::Reverse(task.priority));

    // 执行任务（受并发限制）
    while let Some(task) = queue.pop_front() {
      if running.len() >= MAX_CONCURRENT_TASKS {
        queue.push_front(task);
        break;
      }

      let node_name = task.node_name.clone();
      let handle = Self::execute_task(task.clone()).await;
      running.insert(node_name, RunningTask { task, handle, started_at: now() });
    }
  }

  pub async fn pause_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError> {
    // 实现暂停逻辑
    Ok(())
  }

  pub async fn resume_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError> {
    // 实现恢复逻辑
    Ok(())
  }

  pub async fn cancel_execution(&self, execution_id: &ExecutionId) -> Result<(), WorkflowExecutionError> {
    // 实现取消逻辑
    Ok(())
  }
}

impl TaskScheduler {
  async fn execute_task(
    _task: ExecutionTask,
  ) -> tokio::task::JoinHandle<Result<Vec<ExecutionData>, NodeExecutionError>> {
    tokio::spawn(async move {
      // 执行任务逻辑
      // TODO: 实现具体的任务执行
      Ok(vec![])
    })
  }

  async fn handle_command(
    cmd: SchedulerCommand,
    task_queue: &Arc<RwLock<VecDeque<ExecutionTask>>>,
    waiting_tasks: &Arc<RwLock<HashMap<NodeName, WaitingTask>>>,
    running_tasks: &Arc<RwLock<HashMap<NodeName, RunningTask>>>,
  ) {
    match cmd {
      SchedulerCommand::ScheduleTask(task) => {
        task_queue.write().await.push_back(task);
      }
      SchedulerCommand::CancelTask(node_name) => {
        running_tasks.write().await.remove(&node_name);
      }
      // TODO 处理其他命令...
      _ => {}
    }
  }

  async fn receive_command(
    control_rx: &Arc<RwLock<mpsc::UnboundedReceiver<SchedulerCommand>>>,
  ) -> Option<SchedulerCommand> {
    let mut rx = control_rx.write().await;
    rx.recv().await
  }

  async fn check_task_timeouts(running_tasks: &Arc<RwLock<HashMap<NodeName, RunningTask>>>) {
    let mut tasks = running_tasks.write().await;
    let now = now();

    tasks.retain(|_, running_task| {
      let timeout = running_task.task.timeout.unwrap_or_else(|| Duration::from_secs(300));
      let delta = now - running_task.started_at;
      delta.num_seconds() < timeout.as_secs() as i64
    });
  }
}
