use fusion_server::ctx::CtxW;
use ultimate::Result;
use ultimate_api::v1::PagePayload;

use crate::service::{
  process_definition::process_definition_bmc::ProcessDefinitionBmc,
  process_trigger_rel::{ProcessTriggerRelBmc, ProcessTriggerRelForCreate},
};

use super::{ProcessDefinition, ProcessDefinitionForCreate, SchedProcessForPage};

pub struct ProcessDefinitionSvc;

impl ProcessDefinitionSvc {
  pub async fn find_by_id(ctx: &CtxW, id: i64) -> Result<ProcessDefinition> {
    let entity = ProcessDefinitionBmc::find_by_id(ctx.mm(), id).await?;
    Ok(entity)
  }

  pub async fn create(ctx: &CtxW, entity_c: ProcessDefinitionForCreate, rel_triggers: Vec<i64>) -> Result<i64> {
    let mm = ctx.mm().get_or_clone_with_txn();
    mm.dbx().begin_txn().await?;

    let process_id = ProcessDefinitionBmc::create(&mm, entity_c).await?;

    if !rel_triggers.is_empty() {
      let data =
        rel_triggers.into_iter().map(|trigger_id| ProcessTriggerRelForCreate { process_id, trigger_id }).collect();
      ProcessTriggerRelBmc::insert_many(&mm, data).await?;
    }

    mm.dbx().commit_txn().await?;
    Ok(process_id)
  }

  pub async fn page(ctx: &CtxW, for_page: SchedProcessForPage) -> Result<PagePayload<ProcessDefinition>> {
    let page = ProcessDefinitionBmc::page(ctx.mm(), for_page.filter, for_page.pagination).await?;
    Ok(page)
  }
}
