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

  pub async fn create(
    ctx: &CtxW,
    entity_c: ProcessDefinitionForCreate,
    rel_triggers: Option<Vec<i64>>,
  ) -> Result<i64> {
    let mm = ctx.mm().get_or_clone_with_txn();
    mm.dbx().begin_txn().await?;

    let process_id = ProcessDefinitionBmc::create(&mm, entity_c).await?;

    if let Some(trigger_ids) = rel_triggers {
      if !trigger_ids.is_empty() {
        ProcessTriggerRelBmc::delete_by_job_id(&mm, process_id).await?;
      }

      let data =
        trigger_ids.into_iter().map(|trigger_id| ProcessTriggerRelForCreate { process_id, trigger_id }).collect();
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
