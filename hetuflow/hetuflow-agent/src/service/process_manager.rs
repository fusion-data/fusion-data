use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use hetuflow_core::protocol::{
  ProcessEvent, ProcessEventType, ProcessInfo, ProcessStatus, ResourceLimits, ResourceUsage, ResourceViolation,
  ResourceViolationType,
};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio::sync::{RwLock, broadcast, mpsc};
use ultimate_common::time::now_offset;
use ultimate_core::DataError;
use uuid::Uuid;

use crate::setting::ProcessConfig;

/// 进程管理器。负责管理活跃进程、处理进程事件和后台清理僵尸进程
#[derive(Debug)]
pub struct ProcessManager {
  /// 配置
  process_config: Arc<ProcessConfig>,
  shutdown_rx: Arc<broadcast::Receiver<()>>,
  /// 活跃进程映射
  active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
  /// 进程句柄映射
  process_handles: Arc<RwLock<HashMap<Uuid, Child>>>,
  /// 事件发送器
  event_sender: mpsc::UnboundedSender<ProcessEvent>,
  /// 事件接收器
  event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProcessEvent>>>>,
  /// 资源监控器
  resource_monitor: ResourceMonitor,
}

impl ProcessManager {
  /// 创建新的进程管理器
  pub fn new(process_config: Arc<ProcessConfig>, shutdown_rx: Arc<broadcast::Receiver<()>>) -> Self {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let resource_monitor = ResourceMonitor::new();

    Self {
      process_config,
      shutdown_rx,
      active_processes: Arc::new(RwLock::new(HashMap::default())),
      process_handles: Arc::new(RwLock::new(HashMap::default())),
      event_sender,
      event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
      resource_monitor,
    }
  }

  /// 启动进程管理器
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting ProcessManager");

    // 启动事件处理循环
    let event_receiver = {
      let mut receiver_guard = self.event_receiver.write().await;
      receiver_guard.take()
    };

    if let Some(receiver) = event_receiver {
      let active_processes = Arc::clone(&self.active_processes);
      let process_handles = Arc::clone(&self.process_handles);

      tokio::spawn(async move {
        Self::event_loop(receiver, active_processes, process_handles).await;
      });
    }

    // 启动清理循环
    let cleanup_active_processes = Arc::clone(&self.active_processes);
    let cleanup_process_handles = Arc::clone(&self.process_handles);
    let cleanup_config = self.process_config.clone();
    let cleanup_event_sender = self.event_sender.clone();

    tokio::spawn(async move {
      Self::cleanup_loop(cleanup_active_processes, cleanup_process_handles, cleanup_config, cleanup_event_sender).await;
    });

    info!("ProcessManager started successfully");
    Ok(())
  }

  /// 启动新进程
  pub async fn spawn_process(
    &self,
    task_id: Uuid,
    command: &str,
    args: &[String],
    working_dir: Option<&str>,
    environment: Option<&HashMap<String, String>>,
    resource_limits: Option<ResourceLimits>,
  ) -> Result<Uuid, DataError> {
    let process_id = Uuid::new_v4();

    // 检查并发进程数限制
    {
      let active_processes = self.active_processes.read().await;
      if active_processes.len() >= self.process_config.max_concurrent_processes {
        return Err(DataError::server_error("Maximum concurrent processes limit reached"));
      }
    }

    debug!("Spawning process: {} with args: {:?}, working_dir: {:?}", command, args, working_dir);

    // 构建命令
    let mut cmd = Command::new(command);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());

    // 设置工作目录
    if let Some(dir) = working_dir {
      cmd.current_dir(dir);
    }

    // 设置环境变量
    if let Some(env) = environment {
      for (key, value) in env {
        cmd.env(key, value);
      }
    }

    // 设置进程组（Unix系统）
    #[cfg(unix)]
    {
      use std::os::unix::process::CommandExt;
      cmd.process_group(0);
    }

    // 启动进程
    let mut child = cmd.spawn()?;
    let pid = child.id();

    // 创建进程信息
    let process_info = ProcessInfo {
      pid,
      task_id,
      instance_id: todo!(),
      status: todo!(),
      started_at: todo!(),
      completed_at: todo!(),
      exit_code: todo!(),
      resource_usage: todo!(),
      is_daemon: todo!(),
    };

    // 存储进程信息和句柄
    {
      let mut active_processes = self.active_processes.write().await;
      active_processes.insert(process_id, process_info);
    }
    {
      let mut process_handles = self.process_handles.write().await;
      process_handles.insert(process_id, child);
    }

    // 发送进程启动事件
    let _ = self.event_sender.send(ProcessEvent {
      pid: 0, // TODO: process_id,
      task_id,
      event_type: ProcessEventType::Started,
      timestamp: now_offset(),
      data: None,
    });

    info!("Process spawned successfully: {} (PID: {})", process_id, pid);
    Ok(process_id)
  }

  /// 强制终止进程
  pub async fn kill_process(&self, process_id: Uuid) -> Result<(), DataError> {
    debug!("Killing process: {}", process_id);

    let mut child_opt = {
      let mut process_handles = self.process_handles.write().await;
      process_handles.remove(&process_id)
    };

    if let Some(mut child) = child_opt {
      // 尝试优雅终止
      #[cfg(unix)]
      {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        if child.id() > 0 {
          let pid = child.id();
          let pid = Pid::from_raw(pid as i32);
          // 发送 SIGTERM
          let _ = signal::kill(pid, Signal::SIGTERM);

          // 等待一段时间
          tokio::time::sleep(Duration::from_secs(5)).await;

          // 如果还在运行，发送 SIGKILL
          match child.try_wait() {
            Ok(Some(_)) => {
              debug!("Process {} terminated gracefully", process_id);
            }
            Ok(None) => {
              warn!("Process {} did not terminate gracefully, sending SIGKILL", process_id);
              let _ = signal::kill(pid, Signal::SIGKILL);
              let _ = child.wait();
            }
            Err(e) => {
              error!("Error checking process status: {}", e);
              let _ = child.kill();
            }
          }
        }
      }

      #[cfg(windows)]
      {
        let _ = child.kill();
        let _ = child.wait();
      }

      // 更新进程状态
      {
        let mut active_processes = self.active_processes.write().await;
        if let Some(process_info) = active_processes.get_mut(&process_id) {
          process_info.status = ProcessStatus::Killed;
          process_info.completed_at = Some(now_offset());
        }
      }

      // 发送进程终止事件
      let task_id = {
        let active_processes = self.active_processes.read().await;
        active_processes.get(&process_id).map(|p| p.task_id)
      };

      if let Some(task_id) = task_id {
        let _ = self.event_sender.send(ProcessEvent {
          pid: 0, // TODO: process_id,
          task_id,
          event_type: ProcessEventType::Killed,
          timestamp: now_offset(),
          data: None,
        });
      }

      info!("Process killed successfully: {}", process_id);
    } else {
      warn!("Process not found or already terminated: {}", process_id);
    }

    Ok(())
  }

  /// 获取进程信息
  pub async fn get_process_info(&self, process_id: Uuid) -> Option<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.get(&process_id).cloned()
  }

  /// 获取所有活跃进程
  pub async fn get_active_processes(&self) -> Vec<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.values().cloned().collect()
  }

  /// 获取任务的进程列表
  pub async fn get_task_processes(&self, task_id: Uuid) -> Vec<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.values().filter(|p| p.task_id == task_id).cloned().collect()
  }

  /// 事件处理循环
  async fn event_loop(
    mut receiver: mpsc::UnboundedReceiver<ProcessEvent>,
    active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    process_handles: Arc<RwLock<HashMap<Uuid, Child>>>,
  ) {
    info!("ProcessManager event loop started");

    // TODO:

    info!("ProcessManager event loop ended");
  }

  /// 清理循环
  async fn cleanup_loop(
    active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    process_handles: Arc<RwLock<HashMap<Uuid, Child>>>,
    config: Arc<ProcessConfig>,
    event_sender: mpsc::UnboundedSender<ProcessEvent>,
  ) {
    info!("ProcessManager cleanup loop started");

    let mut cleanup_interval = tokio::time::interval(config.cleanup_interval);
    let mut zombie_check_interval = tokio::time::interval(config.zombie_check_interval);

    loop {
      tokio::select! {
          _ = cleanup_interval.tick() => {
              Self::cleanup_completed_processes(&active_processes, &process_handles).await;
              Self::cleanup_timeout_processes(&active_processes, &process_handles, &config, &event_sender).await;
          }
          _ = zombie_check_interval.tick() => {
              Self::check_zombie_processes(&active_processes, &process_handles, &event_sender).await;
          }
      }
    }
  }

  /// 清理已完成的进程
  async fn cleanup_completed_processes(
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    process_handles: &Arc<RwLock<HashMap<Uuid, Child>>>,
  ) {
    let mut to_remove = Vec::new();

    {
      let mut handles = process_handles.write().await;
      let mut processes = active_processes.write().await;

      for (process_id, child) in handles.iter_mut() {
        match child.try_wait() {
          Ok(Some(exit_status)) => {
            debug!("Process {} exited with status: {:?}", process_id, exit_status);

            if let Some(process_info) = processes.get_mut(process_id) {
              process_info.status = ProcessStatus::Completed;
              process_info.completed_at = Some(now_offset());
              process_info.exit_code = exit_status.code();
            }

            to_remove.push(*process_id);
          }
          Ok(None) => {
            // 进程仍在运行
          }
          Err(e) => {
            error!("Error checking process status for {}: {}", process_id, e);
            to_remove.push(*process_id);
          }
        }
      }

      // 移除已完成的进程句柄
      for process_id in &to_remove {
        handles.remove(process_id);
      }
    }
  }

  /// 清理超时进程
  async fn cleanup_timeout_processes(
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    process_handles: &Arc<RwLock<HashMap<Uuid, Child>>>,
    config: &Arc<ProcessConfig>,
    event_sender: &mpsc::UnboundedSender<ProcessEvent>,
  ) {
    let timeout_duration = config.process_timeout;
    let now = now_offset();
    let mut to_kill = Vec::new();

    {
      let processes = active_processes.read().await;
      for (process_id, process_info) in processes.iter() {
        if process_info.status == ProcessStatus::Running {
          let elapsed = now - process_info.started_at;
          if elapsed.num_milliseconds() as u128 > timeout_duration.as_millis() {
            to_kill.push((*process_id, process_info.task_id));
          }
        }
      }
    }

    for (process_id, task_id) in to_kill {
      warn!("Process {} timed out, killing it", process_id);

      // 强制终止进程
      {
        let mut handles = process_handles.write().await;
        if let Some(mut child) = handles.remove(&process_id) {
          let _ = child.kill();
          let _ = child.wait();
        }
      }

      // 更新进程状态
      {
        let mut processes = active_processes.write().await;
        if let Some(process_info) = processes.get_mut(&process_id) {
          process_info.status = ProcessStatus::Killed;
          process_info.completed_at = Some(now);
        }
      }

      // 发送超时事件
      let _ = event_sender.send(ProcessEvent {
        pid: 0, // TODO: process_id,
        task_id,
        event_type: ProcessEventType::BecameZombie,
        timestamp: now,
        data: Some(json!({"message": "Process killed due to timeout"})),
      });
    }
  }

  /// 检查僵尸进程
  async fn check_zombie_processes(
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    process_handles: &Arc<RwLock<HashMap<Uuid, Child>>>,
    event_sender: &mpsc::UnboundedSender<ProcessEvent>,
  ) {
    let mut to_cleanup = Vec::new();

    {
      let processes = active_processes.read().await;
      let handles = process_handles.read().await;

      for (process_id, process_info) in processes.iter() {
        if process_info.status == ProcessStatus::Running {
          if Self::is_zombie_process(process_info.pid) {
            to_cleanup.push((*process_id, process_info.task_id));
          }
        }
      }
    }

    for (process_id, task_id) in to_cleanup {
      warn!("Zombie process detected: {}", process_id);
      Self::cleanup_zombie_process(process_id, task_id, active_processes, process_handles, event_sender).await;
    }
  }

  /// 检查是否为僵尸进程
  fn is_zombie_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
      use std::fs;

      let stat_path = format!("/proc/{}/stat", pid);
      if let Ok(stat_content) = fs::read_to_string(stat_path) {
        let fields: Vec<&str> = stat_content.split_whitespace().collect();
        if fields.len() > 2 {
          return fields[2] == "Z";
        }
      }
    }

    #[cfg(windows)]
    {
      // Windows 没有僵尸进程的概念
      return false;
    }

    false
  }

  /// 清理僵尸进程
  async fn cleanup_zombie_process(
    process_id: Uuid,
    task_id: Uuid,
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    process_handles: &Arc<RwLock<HashMap<Uuid, Child>>>,
    event_sender: &mpsc::UnboundedSender<ProcessEvent>,
  ) {
    // 移除进程句柄
    {
      let mut handles = process_handles.write().await;
      handles.remove(&process_id);
    }

    // 更新进程状态
    {
      let mut processes = active_processes.write().await;
      if let Some(process_info) = processes.get_mut(&process_id) {
        process_info.status = ProcessStatus::Zombie;
        process_info.completed_at = Some(now_offset());
      }
    }

    // 发送僵尸进程事件
    let _ = event_sender.send(ProcessEvent {
      pid: 0, // TODO: process_id,
      task_id,
      event_type: ProcessEventType::BecameZombie,
      timestamp: now_offset(),
      data: Some(json!({"message": "Zombie process detected and cleaned up"})),
    });
  }
}

/// 资源监控器
#[derive(Debug)]
pub struct ResourceMonitor {
  /// 监控间隔
  monitor_interval: Duration,
}

impl ResourceMonitor {
  /// 创建新的资源监控器
  pub fn new() -> Self {
    Self { monitor_interval: Duration::from_secs(5) }
  }

  /// 监控进程资源使用情况
  pub async fn monitor_process(
    &self,
    pid: u32,
    limits: &ResourceLimits,
  ) -> Result<Option<ResourceViolation>, DataError> {
    let usage = self.get_resource_usage(pid).await?;

    // 检查内存限制
    if let Some(memory_limit) = limits.max_memory_mb {
      if usage.memory_mb > memory_limit as f64 {
        return Ok(Some(ResourceViolation {
          violation_type: ResourceViolationType::MemoryExceeded,
          current_value: usage.memory_mb,
          limit_value: memory_limit as f64,
          timestamp: now_offset(),
        }));
      }
    }

    // 检查CPU限制
    if let Some(cpu_limit) = limits.max_cpu_percent {
      if usage.cpu_percent > cpu_limit as f64 {
        return Ok(Some(ResourceViolation {
          violation_type: ResourceViolationType::CpuExceeded,
          current_value: usage.cpu_percent,
          limit_value: cpu_limit as f64,
          timestamp: now_offset(),
        }));
      }
    }

    Ok(None)
  }

  /// 获取进程资源使用情况
  async fn get_resource_usage(&self, pid: u32) -> Result<ResourceUsage, DataError> {
    #[cfg(unix)]
    {
      use std::fs;

      // 读取内存使用情况
      let status_path = format!("/proc/{}/status", pid);
      let mut memory_mb = 0.0;

      if let Ok(status_content) = fs::read_to_string(status_path) {
        for line in status_content.lines() {
          if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
              if let Ok(kb) = parts[1].parse::<f64>() {
                memory_mb = kb / 1024.0;
              }
            }
            break;
          }
        }
      }

      // 读取CPU使用情况（简化实现）
      let stat_path = format!("/proc/{}/stat", pid);
      let mut cpu_percent = 0.0;

      if let Ok(stat_content) = fs::read_to_string(stat_path) {
        let fields: Vec<&str> = stat_content.split_whitespace().collect();
        if fields.len() > 15 {
          // 简化的CPU使用率计算
          if let (Ok(utime), Ok(stime)) = (fields[13].parse::<u64>(), fields[14].parse::<u64>()) {
            let total_time = utime + stime;
            // 这里应该有更复杂的CPU使用率计算逻辑
            // 为了简化，我们使用一个估算值
            cpu_percent = (total_time as f64) / 100.0; // 简化计算
          }
        }
      }

      Ok(ResourceUsage { memory_mb, cpu_percent, runtime_secs: 0, output_size_bytes: 0 })
    }

    #[cfg(windows)]
    {
      // Windows 平台的资源监控实现
      // TODO: 这里需要使用 Windows API 或第三方库，为了简化，返回默认值
      Ok(ResourceUsage { memory_mb: 0.0, cpu_percent: 0.0, disk_io_mb: 0.0, network_io_mb: 0.0 })
    }
  }
}
