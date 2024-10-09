use fusion_server::ctx::CtxW;
use ultimate::Result;
use ultimate_api::v1::PagePayload;

use super::{trigger_definition_bmc::TriggerDefinitionBmc, TriggerDefinition, TriggerDefinitionForPage};

pub struct TriggerDefinitionSvc;

impl TriggerDefinitionSvc {
  pub async fn page(ctx: &CtxW, for_page: TriggerDefinitionForPage) -> Result<PagePayload<TriggerDefinition>> {
    let page = TriggerDefinitionBmc::page(ctx.mm(), for_page.filter, for_page.pagination).await?;
    Ok(page)
  }
}
