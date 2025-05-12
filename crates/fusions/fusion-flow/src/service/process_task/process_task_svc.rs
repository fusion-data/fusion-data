use fusiondata_context::ctx::CtxW;
use ultimate_core::Result;

use super::{JobTaskForPage, JobTaskPage};

pub struct ProcessTaskSvc;

impl ProcessTaskSvc {
  pub async fn page(ctx: &CtxW, for_page: JobTaskForPage) -> Result<JobTaskPage> {
    todo!()
  }
}
