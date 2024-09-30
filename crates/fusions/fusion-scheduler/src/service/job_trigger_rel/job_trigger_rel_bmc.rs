use modql::filter::OpValValue;
use ultimate_db::{
  base::{self, DbBmc},
  ModelManager, Result,
};
use uuid::Uuid;

use super::{JobTriggerRel, JobTriggerRelFilter, JobTriggerRelForCreate};

pub struct JobTriggerRelBmc;
impl DbBmc for JobTriggerRelBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_job_trigger_rel";

  fn has_modification_timestamps() -> bool {
    false
  }
}

impl JobTriggerRelBmc {
  pub async fn delete_by_job_id(mm: &ModelManager, job_id: Uuid) -> Result<u64> {
    let size = base::delete::<Self, _>(
      mm,
      JobTriggerRelFilter { job_id: Some(OpValValue::Eq(serde_json::to_value(job_id)?).into()), ..Default::default() },
    )
    .await?;
    Ok(size)
  }

  pub async fn insert_many(mm: &ModelManager, data: impl IntoIterator<Item = JobTriggerRelForCreate>) -> Result<u64> {
    base::insert_many::<Self, _>(mm, data).await
  }

  pub async fn find_many(mm: &ModelManager, filter: JobTriggerRelFilter) -> Result<Vec<JobTriggerRel>> {
    base::find_many::<Self, _, _>(mm, filter, None).await
  }
}
