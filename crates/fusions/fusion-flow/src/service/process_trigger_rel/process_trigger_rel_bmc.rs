use modelsql::{
  base::{self, DbBmc},
  filter::OpValUuid,
  ModelManager, Result, SqlError,
};
use uuid::Uuid;

use super::{ProcessTriggerRel, ProcessTriggerRelFilter, ProcessTriggerRelForCreate};

pub struct ProcessTriggerRelBmc;
impl DbBmc for ProcessTriggerRelBmc {
  const TABLE: &'static str = "process_trigger_rel";

  fn has_modification_timestamps() -> bool {
    false
  }
}

impl ProcessTriggerRelBmc {
  pub async fn delete_by(mm: &ModelManager, process_id: Option<Uuid>, trigger_id: Option<Uuid>) -> Result<u64> {
    if process_id.is_none() && trigger_id.is_none() {
      return Err(SqlError::InvalidArgument {
        message: "At least one of 'process_id' and 'trigger_id' is required".to_string(),
      });
    }
    let size = base::delete::<Self, _>(
      mm,
      ProcessTriggerRelFilter {
        process_id: process_id.map(|pid| OpValUuid::Eq(pid).into()),
        trigger_id: trigger_id.map(|tid| OpValUuid::Eq(tid).into()),
        ..Default::default()
      },
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
