use fusion_server::ctx::CtxW;
use ultimate::Result;
use ultimate_api::v1::PagePayload;

use super::{trigger_bmc::TriggerBmc, SchedTrigger, SchedTriggerForPage};

pub struct TriggerSvc;

impl TriggerSvc {
  pub async fn page(ctx: &CtxW, for_page: SchedTriggerForPage) -> Result<PagePayload<SchedTrigger>> {
    let page = TriggerBmc::page(ctx.mm(), for_page.filter, for_page.pagination).await?;
    Ok(page)
  }
}
