use fusion_server::ctx::CtxW;
use ultimate::Result;
use uuid::Uuid;

pub struct SchedulerSvc;

impl SchedulerSvc {
  /// 手动触发 Job
  ///
  /// 触发成功返回 job_task_id
  pub async fn trigger_job(ctx: &CtxW, job_id: Uuid) -> Result<Uuid> {
    todo!()
  }
}
