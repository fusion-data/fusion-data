use fusiondata_context::ctx::CtxW;
use ultimate_core::Result;

use super::{JobTaskForPage, JobTaskPage};

pub struct ProcessTaskSvc;

impl ProcessTaskSvc {
  pub async fn page(_ctx: &CtxW, _for_page: JobTaskForPage) -> Result<JobTaskPage> {
    todo!()
  }
}
