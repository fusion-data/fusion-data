use fusion_server::ctx::CtxW;
use ultimate::Result;

use super::{JobTaskForPage, JobTaskPage};

pub struct JobTaskSvc;

impl JobTaskSvc {
  pub async fn page(ctx: &CtxW, for_page: JobTaskForPage) -> Result<JobTaskPage> {
    todo!()
  }
}
