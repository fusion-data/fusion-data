use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use fusion_common::ahash::HashMap;
use fusion_common::time::now_offset;
use fusion_core::DataError;
use hetuflow_core::protocol::{
  ProcessEvent, ProcessEventType, ProcessInfo, ProcessStatus, ResourceUsage, ResourceViolation, ResourceViolationType,
};
use hetuflow_core::types::ResourceLimits;
use log::{debug, error, info, warn};
use mea::shutdown::ShutdownRecv;
use serde_json::json;
use tokio::process::Command;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::setting::ProcessConfig;

/// 进程管理器。负责管理活跃进程、处理进程事件和后台清理僵尸进程
#[derive(Debug)]
pub struct ProcessManager {
  /// 配置
  process_config: Arc<ProcessConfig>,
  _shutdown_rx: ShutdownRecv,
  /// 活跃进程映射
  active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
  /// 事件广播发送器
  event_broadcaster: broadcast::Sender<ProcessEvent>,
  /// 资源监控器
  _resource_monitor: ResourceMonitor,
}

impl ProcessManager {
  /// 创建新的进程管理器
  pub fn new(process_config: Arc<ProcessConfig>, shutdown_rx: ShutdownRecv) -> Self {
    let (event_broadcaster, _) = broadcast::channel(1000);
    let resource_monitor = ResourceMonitor::new();

    Self {
      process_config,
      _shutdown_rx: shutdown_rx,
      active_processes: Arc::new(RwLock::new(HashMap::default())),
      event_broadcaster,
      _resource_monitor: resource_monitor,
    }
  }

  /// 启动进程管理器
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting ProcessManager");

    // 启动清理循环
    let cleanup_active_processes = Arc::clone(&self.active_processes);
    let cleanup_config = self.process_config.clone();
    let cleanup_event_broadcaster = self.event_broadcaster.clone();

    tokio::spawn(async move {
      Self::cleanup_loop(cleanup_active_processes, cleanup_config, cleanup_event_broadcaster).await;
    });

    info!("ProcessManager started successfully");
    Ok(())
  }

  /// 启动新进程
  pub async fn spawn_process(
    &self,
    instance_id: Uuid,
    cmd: &str,
    args: &[String],
    working_dir: Option<&str>,
    environment: Option<&HashMap<String, String>>,
    _resource_limits: Option<&ResourceLimits>,
  ) -> Result<Uuid, DataError> {
    let process_id = Uuid::now_v7();
    // 检查并发进程数限制
    {
      let active_processes = self.active_processes.read().await;
      if active_processes.len() >= self.process_config.max_concurrent_processes {
        return Err(DataError::server_error("Maximum concurrent processes limit reached"));
      }
    }

    debug!("Spawning process: {} with args: {:?}, working_dir: {:?}", cmd, args, working_dir);

    // 构建命令
    let mut cmd = Command::new(cmd);
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
      cmd.process_group(0);
    }

    // 启动进程
    let child = cmd.spawn()?;
    let pid: u32 = child.id().ok_or_else(|| DataError::server_error("Failed to get process ID"))?;

    // 创建进程信息
    let process_info = ProcessInfo {
      pid,
      instance_id,
      status: ProcessStatus::Running,
      started_at: now_offset(),
      completed_at: None,
      exit_code: None,
      resource_usage: None,
      is_daemon: false,
      child: Arc::new(tokio::sync::Mutex::new(child)),
    };

    // 存储进程信息
    {
      let mut active_processes = self.active_processes.write().await;
      active_processes.insert(process_id, process_info);
    }

    // 发送进程启动事件
    let _ = self.event_broadcaster.send(ProcessEvent {
      process_id,
      instance_id,
      event_type: ProcessEventType::Started,
      timestamp: now_offset(),
      data: None,
    });

    info!("Process spawned successfully: ProcessID: {}, PID {}", process_id, pid);
    Ok(process_id)
  }

  /// 强制终止进程
  pub async fn kill_process(&self, process_id: &Uuid) -> Result<(), DataError> {
    debug!("Killing process: {}", process_id);

    let child_opt = {
      let active_processes = self.active_processes.read().await;
      active_processes.get(process_id).map(|info| info.child.clone())
    };

    if let Some(child_arc) = child_opt {
      // 尝试优雅终止
      #[cfg(unix)]
      {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let child_pid = {
          let child = child_arc.lock().await;
          child.id()
        };

        if let Some(child_pid) = child_pid
          && child_pid > 0
        {
          let pid = Pid::from_raw(child_pid as i32);
          // 发送 SIGTERM
          let _ = signal::kill(pid, Signal::SIGTERM);

          // 等待一段时间
          tokio::time::sleep(Duration::from_secs(5)).await;

          // 如果还在运行，发送 SIGKILL
          let still_running = {
            let mut child = child_arc.lock().await;
            match child.try_wait() {
              Ok(Some(_)) => {
                debug!("Process {} terminated gracefully", process_id);
                false
              }
              Ok(None) => true,
              Err(_) => false,
            }
          };

          if still_running {
            warn!("Process {} did not terminate gracefully, sending SIGKILL", process_id);
            let _ = signal::kill(pid, Signal::SIGKILL);
          }
        }

        // 等待进程结束，但不持有锁
        let _wait_result = {
          let mut child = child_arc.lock().await;
          child.try_wait()
        };
      }

      #[cfg(windows)]
      {
        {
          let mut child = child_arc.lock().unwrap();
          let _ = child.kill();
        }

        // 等待进程结束，但不持有锁
        loop {
          let wait_result = {
            let mut child = child_arc.lock().unwrap();
            child.try_wait()
          };

          match wait_result {
            Ok(Some(_)) => break,
            Ok(None) => {
              tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(_) => break,
          }
        }
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
      let process_instance_id = {
        let active_processes = self.active_processes.read().await;
        active_processes.get(process_id).map(|p| p.instance_id)
      };

      if let Some(instance_id) = process_instance_id {
        let _ = self.event_broadcaster.send(ProcessEvent {
          process_id: *process_id,
          instance_id,
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
  pub async fn get_process_info(&self, process_id: &Uuid) -> Option<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.get(process_id).cloned()
  }

  /// 获取所有活跃进程
  pub async fn get_active_processes(&self) -> Vec<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.values().cloned().collect()
  }

  /// 获取事件接收器
  pub fn subscribe_events(&self) -> broadcast::Receiver<ProcessEvent> {
    self.event_broadcaster.subscribe()
  }

  /// 清理循环
  async fn cleanup_loop(
    active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    config: Arc<ProcessConfig>,
    event_broadcaster: broadcast::Sender<ProcessEvent>,
  ) {
    info!("ProcessManager cleanup loop started");

    let mut cleanup_interval = tokio::time::interval(config.cleanup_interval);
    let mut zombie_check_interval = tokio::time::interval(config.zombie_check_interval);

    loop {
      tokio::select! {
          _ = cleanup_interval.tick() => {
              Self::cleanup_completed_processes(&active_processes).await;
              Self::cleanup_timeout_processes(&active_processes, &config, &event_broadcaster).await;
          }
          _ = zombie_check_interval.tick() => {
              Self::check_zombie_processes(&active_processes, &event_broadcaster).await;
          }
      }
    }
  }

  /// 清理已完成的进程
  async fn cleanup_completed_processes(active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>) {
    let mut to_remove = Vec::new();

    {
      let mut processes = active_processes.write().await;

      for (process_id, process_info) in processes.iter_mut() {
        let mut child = process_info.child.lock().await;
        match child.try_wait() {
          Ok(Some(exit_status)) => {
            debug!("Process {} exited with status: {:?}", process_id, exit_status);

            process_info.status = ProcessStatus::Completed;
            process_info.completed_at = Some(now_offset());
            process_info.exit_code = exit_status.code();

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
    }

    // 移除已完成的进程
    {
      let mut processes = active_processes.write().await;
      for process_id in &to_remove {
        processes.remove(process_id);
      }
    }
  }

  /// 清理超时进程
  async fn cleanup_timeout_processes(
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    config: &Arc<ProcessConfig>,
    event_broadcaster: &broadcast::Sender<ProcessEvent>,
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
            to_kill.push((*process_id, process_info.instance_id));
          }
        }
      }
    }

    for (process_id, instance_id) in to_kill {
      warn!("Process {} timed out, killing it", process_id);

      // 强制终止进程并更新状态
      let child_arc = {
        let processes = active_processes.read().await;
        processes.get(&process_id).map(|info| info.child.clone())
      };

      if let Some(child_arc) = child_arc {
        // 终止进程，避免跨 await 持有锁
        {
          let mut child = child_arc.lock().await;
          let _ = child.kill().await;
          let _ = child.wait().await;
        }

        // 更新进程状态
        {
          let mut processes = active_processes.write().await;
          if let Some(process_info) = processes.get_mut(&process_id) {
            process_info.status = ProcessStatus::Killed;
            process_info.completed_at = Some(now);
          }
        }
      }

      // 发送超时事件
      let _ = event_broadcaster.send(ProcessEvent {
        process_id,
        instance_id,
        event_type: ProcessEventType::BecameZombie,
        timestamp: now,
        data: Some(json!({"message": "Process killed due to timeout"})),
      });
    }
  }

  /// 检查僵尸进程
  async fn check_zombie_processes(
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    event_broadcaster: &broadcast::Sender<ProcessEvent>,
  ) {
    let mut to_cleanup = Vec::new();

    {
      let processes = active_processes.read().await;

      for (process_id, process_info) in processes.iter() {
        if process_info.status == ProcessStatus::Running && Self::is_zombie_process(process_info.pid) {
          to_cleanup.push((*process_id, process_info.instance_id));
        }
      }
    }

    for (process_id, instance_id) in to_cleanup {
      warn!("Zombie process detected: {}", process_id);
      Self::cleanup_zombie_process(&process_id, instance_id, active_processes, event_broadcaster).await;
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
    process_id: &Uuid,
    instance_id: Uuid,
    active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    event_broadcaster: &broadcast::Sender<ProcessEvent>,
  ) {
    // 更新进程状态
    {
      let mut processes = active_processes.write().await;
      if let Some(process_info) = processes.get_mut(&process_id) {
        process_info.status = ProcessStatus::Zombie;
        process_info.completed_at = Some(now_offset());
      }
    }

    // 发送僵尸进程事件
    let _ = event_broadcaster.send(ProcessEvent {
      process_id: *process_id,
      instance_id,
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
    if let Some(memory_limit) = limits.max_memory_mb
      && usage.memory_mb > memory_limit as f64
    {
      return Ok(Some(ResourceViolation {
        violation_type: ResourceViolationType::MemoryExceeded,
        current_value: usage.memory_mb,
        limit_value: memory_limit as f64,
        timestamp: now_offset(),
      }));
    }

    // 检查CPU限制
    if let Some(cpu_limit) = limits.max_cpu_percent
      && usage.cpu_percent > cpu_limit as f64
    {
      return Ok(Some(ResourceViolation {
        violation_type: ResourceViolationType::CpuExceeded,
        current_value: usage.cpu_percent,
        limit_value: cpu_limit as f64,
        timestamp: now_offset(),
      }));
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
            if parts.len() >= 2
              && let Ok(kb) = parts[1].parse::<f64>()
            {
              memory_mb = kb / 1024.0;
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
