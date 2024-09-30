use fusion_server::ctx::CtxW;
use ultimate::Result;
use ultimate_api::v1::PagePayload;
use uuid::Uuid;

use crate::service::{
  job::job_bmc::SchedJobBmc,
  job_trigger_rel::{JobTriggerRelBmc, JobTriggerRelForCreate},
};

use super::{SchedJob, SchedJobForCreate, SchedJobForPage};

pub struct JobSvc;

impl JobSvc {
  pub async fn create(ctx: &CtxW, mut entity_c: SchedJobForCreate, rel_triggers: Option<Vec<Uuid>>) -> Result<Uuid> {
    if entity_c.id.is_none() {
      entity_c.id = Some(Uuid::now_v7());
    }
    let job_id = entity_c.id.unwrap();

    let mm = ctx.mm().get_or_clone_with_txn()?;
    mm.dbx().begin_txn().await?;

    SchedJobBmc::insert(&mm, entity_c).await?;

    if let Some(trigger_ids) = rel_triggers {
      if !trigger_ids.is_empty() {
        JobTriggerRelBmc::delete_by_job_id(&mm, job_id).await?;
      }
      JobTriggerRelBmc::insert_many(
        &mm,
        trigger_ids.into_iter().map(|trigger_id| JobTriggerRelForCreate { job_id, trigger_id }),
      )
      .await?;
    }

    mm.dbx().commit_txn().await?;
    Ok(job_id)
  }

  pub async fn page(ctx: &CtxW, for_page: SchedJobForPage) -> Result<PagePayload<SchedJob>> {
    let page = SchedJobBmc::page(ctx.mm(), for_page.filter, for_page.pagination).await?;
    Ok(page)
  }
}
