use modql::filter::OpValValue;
use ultimate_db::{
  base::{self, DbBmc},
  ModelManager, Result,
};
use uuid::Uuid;

use super::{JobTrigger, JobTriggerFilter, JobTriggerForCreate};

pub struct JobTriggerBmc;
impl DbBmc for JobTriggerBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_job_trigger";
}

impl JobTriggerBmc {
  pub async fn delete_by_job_id(mm: &ModelManager, job_id: Uuid) -> Result<u64> {
    let size = base::delete::<Self, _>(
      mm,
      JobTriggerFilter { job_id: Some(OpValValue::Eq(serde_json::to_value(job_id)?).into()), ..Default::default() },
    )
    .await?;
    Ok(size)
  }

  pub async fn insert_many(mm: &ModelManager, data: impl IntoIterator<Item = JobTriggerForCreate>) -> Result<u64> {
    base::insert_many::<Self, _>(mm, data).await
  }

  pub async fn find_many(mm: &ModelManager, filter: JobTriggerFilter) -> Result<Vec<JobTrigger>> {
    base::find_many::<Self, _, _>(mm, filter, None).await
  }
}
