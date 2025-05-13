use fusiondata_context::ctx::CtxW;
use ultimate_core::Result;
use uuid::Uuid;

pub struct SchedulerSvc;

impl SchedulerSvc {
  /// 手动触发 Job
  ///
  /// 触发成功返回 job_task_id
  pub async fn trigger_process(_ctx: &CtxW, _process_id: Uuid) -> Result<Uuid> {
    todo!()
  }
}
