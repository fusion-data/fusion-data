use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fusion_common::ahash::HashMap;
use fusion_common::time::now_epoch_millis;
use fusion_core::DataError;
use hetuflow_core::protocol::{
  ProcessEvent, ProcessEventKind, ProcessInfo, ProcessStatus, ResourceUsage, ResourceViolation, ResourceViolationType,
};
use hetuflow_core::types::ResourceLimits;
use log::{debug, error, info, warn};
use mea::shutdown::ShutdownRecv;
use tokio::process::Command;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::setting::ProcessConfig;

/// 进程管理器。负责管理活跃进程、处理进程事件和后台清理僵尸进程
pub struct ProcessManager {
  /// 配置
  process_config: Arc<ProcessConfig>,
  shutdown_rx: Mutex<Option<ShutdownRecv>>,
  /// 活跃进程映射。键为任务实例ID，值为进程信息。
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
      shutdown_rx: Mutex::new(Some(shutdown_rx)),
      active_processes: Arc::new(RwLock::new(HashMap::default())),
      event_broadcaster,
      _resource_monitor: resource_monitor,
    }
  }

  /// 启动进程管理器
  pub fn start(&self) -> Result<Vec<JoinHandle<()>>, DataError> {
    info!("Starting ProcessManager");

    let active_processes = self.active_processes.clone();
    let process_config = self.process_config.clone();
    let event_broadcaster = self.event_broadcaster.clone();
    let shutdown_rx = self.shutdown_rx.lock().unwrap().take().unwrap();

    let handle = tokio::spawn(Self::cleanup_loop(active_processes, process_config, event_broadcaster, shutdown_rx));

    info!("ProcessManager started successfully");
    Ok(vec![handle])
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
    // 检查并发进程数限制
    {
      let active_processes = self.active_processes.read().await;
      if active_processes.len() >= self.process_config.max_concurrent_processes as usize {
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
    let pid: u32 = child.id().ok_or_else(|| DataError::server_error("Start process failed"))?;

    // 创建进程信息
    let started_at = now_epoch_millis();
    let process_info = ProcessInfo {
      pid,
      instance_id,
      status: ProcessStatus::Running,
      started_at,
      completed_at: None,
      exit_code: None,
      child: Arc::new(mea::mutex::Mutex::new(child)),
    };

    // 存储进程信息
    {
      let mut active_processes = self.active_processes.write().await;
      active_processes.insert(instance_id, process_info.clone());
    }

    // 发送进程启动事件
    let _ = self.event_broadcaster.send(ProcessEvent {
      instance_id,
      kind: ProcessEventKind::Started,
      timestamp: started_at,
      data: serde_json::to_string(&process_info).ok(),
    });

    info!("Process spawned successfully: ProcessID: {}, PID {}", instance_id, pid);
    Ok(instance_id)
  }

  /// 强制终止进程
  /// 终止进程，返回 JoinHandle 用于异步处理
  pub fn spawn_kill_process(&self, instance_id: Uuid) -> tokio::task::JoinHandle<Result<Option<Uuid>, DataError>> {
    let active_processes = self.active_processes.clone();
    let event_broadcaster = self.event_broadcaster.clone();
    _spawn_kill_process(instance_id, active_processes, event_broadcaster)
  }

  pub async fn kill_all_processes(&self) -> Result<(), DataError> {
    let handles = self
      .active_processes
      .read()
      .await
      .keys()
      .map(|instance_id| (*instance_id, self.spawn_kill_process(*instance_id)))
      .collect::<Vec<_>>();

    // Wait for all kill process tasks to complete
    for (instance_id, handle) in handles {
      if let Err(e) = handle.await {
        warn!("Error waiting for kill process task for {}, error: {}", instance_id, e);
      }
    }

    Ok(())
  }

  /// 获取进程信息
  pub async fn get_process_info(&self, instance_id: &Uuid) -> Option<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.get(instance_id).cloned()
  }

  /// 获取所有活跃进程
  pub async fn get_active_processes(&self) -> Vec<ProcessInfo> {
    let active_processes = self.active_processes.read().await;
    active_processes.values().cloned().collect()
  }

  /// 获取可用容量
  pub async fn available_capacity(&self) -> u32 {
    let active_processes = self.active_processes.read().await;
    self.process_config.max_concurrent_processes - active_processes.len() as u32
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
    shutdown_rx: ShutdownRecv,
  ) {
    info!("ProcessManager cleanup loop started");

    let mut cleanup_interval = tokio::time::interval(config.cleanup_interval);
    let mut zombie_check_interval = tokio::time::interval(config.zombie_check_interval);

    loop {
      tokio::select! {
          _ = cleanup_interval.tick() => {
              Self::cleanup_completed_processes(&active_processes).await;
              Self::cleanup_timeout_processes(&config, active_processes.clone(), event_broadcaster.clone()).await;
          }
          _ = zombie_check_interval.tick() => {
              Self::cleanup_zombie_processes(active_processes.clone(), event_broadcaster.clone()).await;
          }
          _ = shutdown_rx.is_shutdown() => {
              info!("ProcessManager cleanup loop stopped");
              break;
          }
      }
    }
  }

  /// 清理已完成的进程
  async fn cleanup_completed_processes(active_processes: &Arc<RwLock<HashMap<Uuid, ProcessInfo>>>) {
    let mut to_remove = Vec::new();

    {
      let mut processes = active_processes.write().await;

      for (instance_id, process_info) in processes.iter_mut() {
        let mut child = process_info.child.lock().await;
        match child.try_wait() {
          Ok(Some(exit_status)) => {
            debug!("Process {} exited with status: {:?}", instance_id, exit_status);

            process_info.status = ProcessStatus::Completed;
            process_info.completed_at = Some(now_epoch_millis());
            process_info.exit_code = exit_status.code();

            to_remove.push(*instance_id);
          }
          Ok(None) => {
            // 进程仍在运行
          }
          Err(e) => {
            error!("Error checking process status for {}: {}", instance_id, e);
            to_remove.push(*instance_id);
          }
        }
      }
    }

    // 移除已完成的进程
    {
      let mut processes = active_processes.write().await;
      for instance_id in &to_remove {
        processes.remove(instance_id);
      }
    }
  }

  /// 清理超时进程
  async fn cleanup_timeout_processes(
    config: &Arc<ProcessConfig>,
    active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    event_broadcaster: broadcast::Sender<ProcessEvent>,
  ) {
    let timeout_duration = config.process_timeout;
    let now = now_epoch_millis();

    let processes = active_processes.read().await;
    for process_info in processes.values() {
      if process_info.status == ProcessStatus::Running {
        let elapsed = (now - process_info.started_at) as u128;
        if elapsed > timeout_duration.as_millis() {
          warn!("Process {} timed out, killing it", process_info.instance_id);
          let _handle =
            _spawn_kill_process(process_info.instance_id, active_processes.clone(), event_broadcaster.clone());
        }
      }
    }
  }

  /// 清理僵尸进程
  async fn cleanup_zombie_processes(
    active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    event_broadcaster: broadcast::Sender<ProcessEvent>,
  ) {
    let zombies = {
      let mut processes = active_processes.write().await;
      let (zombies, actives) = std::mem::take(&mut *processes).into_iter().partition(|(_, process_info)| {
        process_info.status == ProcessStatus::Running && is_zombie_process(process_info.pid)
      });
      *processes = actives;
      zombies
    };

    for mut process_info in zombies.into_values() {
      info!("Zombie process detected: {}", process_info.instance_id);
      process_info.status = ProcessStatus::Zombie;
      process_info.completed_at = Some(now_epoch_millis());

      // 发送僵尸进程事件
      let _ = event_broadcaster.send(ProcessEvent {
        instance_id: process_info.instance_id,
        kind: ProcessEventKind::BecameZombie,
        timestamp: now_epoch_millis(),
        data: serde_json::to_string(&process_info).ok(),
      });
    }
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

  // Windows 没有僵尸进程的概念

  false
}

fn _spawn_kill_process(
  instance_id: Uuid,
  active_processes: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
  event_broadcaster: broadcast::Sender<ProcessEvent>,
) -> tokio::task::JoinHandle<Result<Option<Uuid>, DataError>> {
  tokio::spawn(async move {
    debug!("Beginning kill process. instance_id: {}", instance_id);

    let mut process_info = {
      match active_processes.write().await.remove(&instance_id) {
        Some(process_info) => process_info,
        None => {
          warn!("Process not found or already terminated: {}", instance_id);
          return Ok(None);
        }
      }
    };

    let child_arc = process_info.child.clone();
    let mut sigterm_sent = false;
    let mut sigkill_sent = false;
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

        // 发送 SIGTERM 信号
        if let Ok(()) = signal::kill(pid, Signal::SIGTERM) {
          debug!("Sent SIGTERM to process {}", instance_id);
          sigterm_sent = true;
        } else {
          warn!("Failed to send SIGTERM to process {}", instance_id);
        }

        // 循环监听进程状态，最多等待30秒
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(30);

        loop {
          // 检查进程是否已终止
          let process_terminated = {
            let mut child = child_arc.lock().await;
            match child.try_wait() {
              Ok(Some(exit_status)) => {
                debug!("Process {} terminated with status: {:?}", instance_id, exit_status);
                true
              }
              Ok(None) => false,
              Err(e) => {
                warn!("Error checking process {} status: {}", instance_id, e);
                true // 假设进程已终止
              }
            }
          };

          if process_terminated {
            if sigterm_sent && !sigkill_sent {
              info!("Process {} terminated gracefully after SIGTERM", instance_id);
            }
            break;
          }

          // 检查是否超时
          if start_time.elapsed() >= timeout_duration && !sigkill_sent {
            warn!("Process {} did not terminate within 30 seconds, sending SIGKILL", instance_id);

            // 发送 SIGKILL 信号
            if let Ok(()) = signal::kill(pid, Signal::SIGKILL) {
              debug!("Sent SIGKILL to process {}", instance_id);
              sigkill_sent = true;
            } else {
              error!("Failed to send SIGKILL to process {}", instance_id);
              break;
            }
          }

          // 如果已发送 SIGKILL 且超过额外的等待时间，强制退出循环
          if sigkill_sent && start_time.elapsed() >= timeout_duration + Duration::from_secs(5) {
            error!("Process {} did not terminate even after SIGKILL", instance_id);
            break;
          }

          // 短暂等待后再次检查
          tokio::time::sleep(Duration::from_millis(100)).await;
        }
      } else {
        warn!("Invalid or missing PID for process {}", instance_id);
      }
    }

    #[cfg(windows)]
    {
      // Windows 平台直接终止进程
      {
        let mut child = child_arc.lock().await;
        if let Err(e) = child.kill().await {
          error!("Failed to kill process {}: {}", instance_id, e);
        } else {
          debug!("Sent kill signal to process {}", instance_id);
          sigkill_sent = true;

          // 发布 SIGKILL 事件（Windows 没有 SIGTERM）
          let _ = event_broadcaster.send(ProcessEvent {
            instance_id: instance_id,
            instance_id,
            kind: ProcessEventKind::Sigkill,
            timestamp: now_epoch_millis(),
            data: None,
          });
        }
      }

      // 等待进程结束
      let start_time = std::time::Instant::now();
      let timeout_duration = Duration::from_secs(30);

      loop {
        let wait_result = {
          let mut child = child_arc.lock().await;
          child.try_wait()
        };

        match wait_result {
          Ok(Some(exit_status)) => {
            debug!("Process {} terminated with status: {:?}", instance_id, exit_status);
            break;
          }
          Ok(None) => {
            if start_time.elapsed() >= timeout_duration {
              error!("Process {} did not terminate within timeout", instance_id);
              break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
          }
          Err(e) => {
            warn!("Error waiting for process {}: {}", instance_id, e);
            break;
          }
        }
      }
    }

    // 更新进程信息状态
    let completed_at = now_epoch_millis();
    process_info.status = ProcessStatus::Killed;
    process_info.completed_at = Some(completed_at);
    let instance_id = process_info.instance_id;

    info!("Process killed successfully. instance_id: {}", instance_id);
    let kind = if sigkill_sent {
      ProcessEventKind::Sigkill
    } else if sigterm_sent {
      ProcessEventKind::Sigterm
    } else {
      ProcessEventKind::Exited
    };
    let data = serde_json::to_string(&process_info).ok();
    let event = ProcessEvent { instance_id, kind, timestamp: completed_at, data };
    if let Err(e) = event_broadcaster.send(event) {
      warn!("Failed to publish process event. instance_id: {}; error: {}", instance_id, e);
    }

    Ok(Some(instance_id))
  })
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
        timestamp: now_epoch_millis(),
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
        timestamp: now_epoch_millis(),
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
      Ok(ResourceUsage { memory_mb: 0.0, cpu_percent: 0.0, runtime_secs: 0, output_size_bytes: 0 })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;
  use tokio::time::timeout;
  use uuid::Uuid;

  #[tokio::test]
  async fn test_kill_process_returns_join_handle() {
    // 这个测试验证 kill_process 方法返回 JoinHandle
    // 注意：这是一个基本的接口测试，不会实际启动进程

    let config = Arc::new(ProcessConfig {
      cleanup_interval: Duration::from_secs(60),
      zombie_check_interval: Duration::from_secs(30),
      process_timeout: Duration::from_secs(300),
      max_concurrent_processes: 10,
      enable_resource_monitoring: false,
      resource_monitor_interval: Duration::from_secs(30),
      limits: crate::setting::ResourceLimits { max_memory_bytes: None, max_cpu_percent: None },
    });

    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();
    let process_manager = ProcessManager::new(config, shutdown_rx);

    // 测试对不存在进程的 kill_process 调用
    let non_existent_instance_id = Uuid::new_v4();
    let kill_handle = process_manager.spawn_kill_process(non_existent_instance_id);

    // 验证返回的是 JoinHandle
    assert!(std::any::type_name_of_val(&kill_handle).contains("JoinHandle"));

    // 等待任务完成，应该返回 NotFound 错误
    let result = timeout(Duration::from_secs(5), kill_handle).await;
    assert!(result.is_ok(), "Kill process task should complete within timeout");

    let task_result = result.unwrap();
    assert!(task_result.is_ok(), "JoinHandle should not have join errors");

    let kill_result = task_result.unwrap();
    assert!(kill_result.is_err(), "Should return error for non-existent process");

    // 清理
    drop(shutdown_tx);
  }
}
