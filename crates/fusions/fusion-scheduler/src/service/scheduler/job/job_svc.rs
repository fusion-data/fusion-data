use fusion_server::ctx::CtxW;
use ultimate::Result;
use uuid::Uuid;

use crate::service::scheduler::{
  job::job_bmc::SchedJobBmc,
  job_trigger::{JobTriggerBmc, JobTriggerForCreate},
};

use super::SchedJobForCreate;

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
        JobTriggerBmc::delete_by_job_id(&mm, job_id).await?;
      }
      JobTriggerBmc::insert_many(
        &mm,
        trigger_ids.into_iter().map(|trigger_id| JobTriggerForCreate { job_id, trigger_id }),
      )
      .await?;
    }

    mm.dbx().commit_txn().await?;
    Ok(job_id)
  }
}
