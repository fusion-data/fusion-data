use modql::filter::OpValInt64;
use ultimate_db::{
  base::{self, DbBmc},
  ModelManager, Result,
};

use super::{ProcessTriggerRel, ProcessTriggerRelFilter, ProcessTriggerRelForCreate};

pub struct ProcessTriggerRelBmc;
impl DbBmc for ProcessTriggerRelBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "process_trigger_rel";

  fn has_modification_timestamps() -> bool {
    false
  }
}

impl ProcessTriggerRelBmc {
  pub async fn delete_by_job_id(mm: &ModelManager, process_id: i64) -> Result<u64> {
    let size = base::delete::<Self, _>(
      mm,
      ProcessTriggerRelFilter { process_id: Some(OpValInt64::Eq(process_id).into()), ..Default::default() },
    )
    .await?;
    Ok(size)
  }

  pub async fn insert_many(mm: &ModelManager, data: Vec<ProcessTriggerRelForCreate>) -> Result<u64> {
    base::insert_many::<Self, _>(mm, data).await
  }

  pub async fn find_many(mm: &ModelManager, filter: ProcessTriggerRelFilter) -> Result<Vec<ProcessTriggerRel>> {
    base::find_many::<Self, _, _>(mm, filter, None).await
  }
}
