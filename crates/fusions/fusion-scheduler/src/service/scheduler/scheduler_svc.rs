use fusion_server::ctx::CtxW;
use ultimate::Result;

pub struct SchedulerSvc;

impl SchedulerSvc {
  /// 手动触发 Job
  ///
  /// 触发成功返回 job_task_id
  pub async fn trigger_process(ctx: &CtxW, process_id: i64) -> Result<i64> {
    todo!()
  }
}
