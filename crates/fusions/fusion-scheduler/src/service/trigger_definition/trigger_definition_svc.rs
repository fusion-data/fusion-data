use fusion_server::ctx::CtxW;
use modql::filter::OpValInt32;
use ultimate::Result;
use ultimate_api::v1::PagePayload;

use crate::service::trigger_definition::TriggerDefinitionFilter;

use super::{trigger_definition_bmc::TriggerDefinitionBmc, TriggerDefinition, TriggerDefinitionForPage};

pub struct TriggerDefinitionSvc;

impl TriggerDefinitionSvc {
  pub async fn page(ctx: &CtxW, for_page: TriggerDefinitionForPage) -> Result<PagePayload<TriggerDefinition>> {
    TriggerDefinitionBmc::page(ctx.mm(), for_page.filter, for_page.pagination).await.map_err(Into::into)
  }

  pub async fn scan_next_triggers(ctx: &CtxW, namespace_ids: Vec<i32>) -> Result<()> {
    let filters =
      vec![TriggerDefinitionFilter { namespace_id: Some(OpValInt32::In(namespace_ids).into()), ..Default::default() }];
    let triggers = TriggerDefinitionBmc::find_many(ctx.mm(), filters, None).await?;

    todo!()
  }
}
