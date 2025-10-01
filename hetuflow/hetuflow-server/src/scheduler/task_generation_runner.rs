use std::sync::Arc;

use fusion_common::time::now_offset;
use fusion_core::{DataError, concurrent::ServiceTask};
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use fusionsql::ModelManager;
use tokio::time::interval;

use crate::setting::HetuflowSetting;

use super::SchedulerSvc;

/// 任务生成运行器
///
/// 负责：
/// - 根据 Schedule 预生成未来一段时间的 SchedTaskInstance
/// - 基于外部事件或 API 调用按需生成 SchedTaskInstance
pub struct TaskGenerationRunner {
  setting: Arc<HetuflowSetting>,
  mm: ModelManager,
  shutdown_rx: ShutdownRecv,
}

impl ServiceTask<()> for TaskGenerationRunner {
  async fn run_loop(&mut self) -> Result<(), DataError> {
    let task_generation_svc = SchedulerSvc::new(self.mm.clone());
    let mut interval = interval(self.setting.server.job_check_interval);
    let duration = self.setting.server.job_check_duration;

    loop {
      tokio::select! {
        _ = interval.tick() => { /* do nothing */}
        _ = self.shutdown_rx.is_shutdown() => {
          info!("Task TaskGenerationRunner shutting down");
          break;
        }
      }

      let from_time = now_offset();
      let to_time = from_time + duration;

      // 生成定时任务
      if let Err(e) = task_generation_svc.generate_tasks_for_schedule(from_time, to_time).await {
        error!("Task generation failed: {}", e);
      }

      // 生成重试任务
      if let Err(e) = task_generation_svc.generate_retry_tasks().await {
        error!("Retry task generation failed: {}", e);
      }
    }

    Ok(())
  }
}

impl TaskGenerationRunner {
  /// 创建新的调度器服务
  pub fn new(setting: Arc<HetuflowSetting>, mm: ModelManager, shutdown_rx: ShutdownRecv) -> Self {
    Self { setting, mm, shutdown_rx }
  }
}
