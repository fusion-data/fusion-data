use fusion_common::time::now_epoch_millis;
use fusion_core::DataError;
use log::{debug, error, info, warn};
use uuid::Uuid;

use hetuflow_core::protocol::{ProcessEvent, ProcessEventKind, ProcessStatus};
use tokio::{sync::broadcast, time::Duration};

use super::ActiveProcesses;

pub fn spawn_kill_process(
  instance_id: Uuid,
  active_processes: ActiveProcesses,
  event_broadcaster: broadcast::Sender<ProcessEvent>,
) -> tokio::task::JoinHandle<Result<Option<Uuid>, DataError>> {
  tokio::spawn(async move {
    debug!("Beginning kill process. instance_id: {}", instance_id);

    let mut process_item = {
      match active_processes.write().await.remove(&instance_id) {
        Some(process_item) => process_item,
        None => {
          warn!("Process not found or already terminated: {}", instance_id);
          return Ok(None);
        }
      }
    };

    // let child_arc = process_item.child.clone();
    let mut sigterm_sent = false;
    let mut sigkill_sent = false;
    // 尝试优雅终止
    #[cfg(unix)]
    {
      use nix::sys::signal::{self, Signal};
      use nix::sys::wait::WaitStatus;
      use nix::unistd::Pid;

      let child_pid = process_item.info.pid;

      if child_pid > 0 {
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
            let wait_options = nix::sys::wait::WaitPidFlag::WNOHANG;
            let wait_status = nix::sys::wait::waitpid(pid, Some(wait_options));
            match wait_status {
              Ok(WaitStatus::Exited(_pid, code)) => {
                info!("Process {} terminated with status: {:?}", instance_id, code);
                process_item.info.exit_code = Some(code);
                true
              }
              _ => false,
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
    process_item.info.status = ProcessStatus::Killed;
    process_item.info.completed_at = Some(completed_at);
    let instance_id = process_item.info.instance_id;

    info!("Process killed successfully. instance_id: {}", instance_id);
    let kind = if sigkill_sent {
      ProcessEventKind::Sigkill
    } else if sigterm_sent {
      ProcessEventKind::Sigterm
    } else {
      ProcessEventKind::Exited
    };
    let data = serde_json::to_string(&process_item.info).ok();
    if let Err(e) = event_broadcaster.send(ProcessEvent::new_with_data(instance_id, kind, data)) {
      warn!("Failed to publish process event. instance_id: {}; error: {}", instance_id, e);
    }

    Ok(Some(instance_id))
  })
}
