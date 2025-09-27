use std::sync::Arc;

use fusion_common::process::is_zombie_process;
use fusion_common::time::now_epoch_millis;
use fusion_core::{DataError, concurrent::ServiceTask};
use hetuflow_core::protocol::{ProcessEvent, ProcessEventKind, ProcessStatus};
use log::{info, warn};
use mea::shutdown::ShutdownRecv;

use tokio::sync::broadcast;

use crate::setting::ProcessSetting;

use super::{ActiveProcesses, ProcessManager, spawn_kill_process};

pub struct ProcessCleanupRunner {
  config: Arc<ProcessSetting>,
  active_processes: ActiveProcesses,
  event_broadcaster: broadcast::Sender<ProcessEvent>,
  shutdown_rx: ShutdownRecv,
}

impl ServiceTask<()> for ProcessCleanupRunner {
  /// 清理循环
  async fn run_loop(&mut self) -> Result<(), DataError> {
    info!("ProcessManager cleanup loop started");

    let mut cleanup_interval = tokio::time::interval(self.config.cleanup_interval);
    let mut zombie_check_interval = tokio::time::interval(self.config.zombie_check_interval);

    loop {
      tokio::select! {
          _ = self.shutdown_rx.is_shutdown() => {
              info!("ProcessManager cleanup loop stopped");
              break;
          }
          _ = zombie_check_interval.tick() => {
              Self::cleanup_zombie_processes(self.active_processes.clone(), self.event_broadcaster.clone()).await;
          }
          _ = cleanup_interval.tick() => {
              Self::cleanup_timeout_processes(&self.config, self.active_processes.clone(), self.event_broadcaster.clone()).await;
          }
      }
    }
    Ok(())
  }
}

impl ProcessCleanupRunner {
  pub fn new(process_manager: Arc<ProcessManager>, shutdown_rx: ShutdownRecv) -> Self {
    Self {
      config: process_manager.config.clone(),
      active_processes: process_manager.active_processes.clone(),
      event_broadcaster: process_manager.event_broadcaster.clone(),
      shutdown_rx,
    }
  }

  /// 清理超时进程
  async fn cleanup_timeout_processes(
    config: &Arc<ProcessSetting>,
    active_processes: ActiveProcesses,
    event_broadcaster: broadcast::Sender<ProcessEvent>,
  ) {
    let timeout_duration = config.process_timeout;
    let now = now_epoch_millis();

    let processes = active_processes.read().await;
    for process_item in processes.values() {
      if process_item.info.status == ProcessStatus::Running {
        let elapsed = (now - process_item.info.started_at) as u128;
        if elapsed > timeout_duration.as_millis() {
          warn!("Process {} timed out, killing it", process_item.info.instance_id);
          let _handle =
            spawn_kill_process(process_item.info.instance_id, active_processes.clone(), event_broadcaster.clone());
        }
      }
    }
  }

  /// 清理僵尸进程
  async fn cleanup_zombie_processes(
    active_processes: ActiveProcesses,
    event_broadcaster: broadcast::Sender<ProcessEvent>,
  ) {
    let zombies = {
      let mut processes = active_processes.write().await;
      let (zombies, actives) = std::mem::take(&mut *processes).into_iter().partition(|(_, process_item)| {
        process_item.info.status == ProcessStatus::Running && is_zombie_process(process_item.info.pid)
      });
      *processes = actives;
      zombies
    };

    for mut process_item in zombies.into_values() {
      info!("Zombie process detected: {}", process_item.info.instance_id);
      process_item.info.status = ProcessStatus::Zombie;
      process_item.info.completed_at = Some(now_epoch_millis());

      // 发送僵尸进程事件
      let _ = event_broadcaster.send(ProcessEvent::new_with_data(
        process_item.info.instance_id,
        ProcessEventKind::BecameZombie,
        serde_json::to_string(&process_item.info).ok(),
      ));
    }
  }
}
