use fusionsql::page::OrderBys;
use fusionsql::{
  ModelManager, SqlError,
  base::DbBmc,
  filter::{OpValInt32, OpValString, OpValUuid},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use uuid::Uuid;

use hetuflow_core::types::JobStatus;

use hetuflow_core::models::{JobFilter, JobForCreate, JobForUpdate, SchedJob};

/// JobBmc 实现
pub struct JobBmc;

impl DbBmc for JobBmc {
  const TABLE: &str = "sched_job";
  const ID_GENERATED_BY_DB: bool = false;
  fn _has_created_by() -> bool {
    false
  }
  fn _has_updated_by() -> bool {
    false
  }
  fn _default_order_bys() -> Option<OrderBys> {
    Some("!id".into())
  }
}

generate_pg_bmc_common!(
  Bmc: JobBmc,
  Entity: SchedJob,
  ForUpdate: JobForUpdate,
  ForInsert: JobForCreate,
);

generate_pg_bmc_filter!(
  Bmc: JobBmc,
  Entity: SchedJob,
  Filter: JobFilter,
);

impl JobBmc {
  /// 查找启用的作业
  pub async fn find_enabled_jobs(mm: &ModelManager) -> Result<Vec<SchedJob>, SqlError> {
    let filter = vec![JobFilter { status: Some(OpValInt32::eq(JobStatus::Enabled as i32)), ..Default::default() }];

    Self::find_many(mm, filter, None).await
  }

  pub async fn find_enabled_by_id(mm: &ModelManager, id: Uuid) -> Result<Option<SchedJob>, SqlError> {
    let filter = vec![JobFilter {
      id: Some(OpValUuid::eq(id)),
      status: Some(OpValInt32::eq(JobStatus::Enabled as i32)),
      ..Default::default()
    }];

    Self::find_unique(mm, filter).await
  }

  /// 根据命名空间查找作业
  pub async fn find_jobs_by_namespace(mm: &ModelManager, namespace_id: &str) -> Result<Vec<SchedJob>, SqlError> {
    let filter = vec![JobFilter { namespace_id: Some(OpValString::eq(namespace_id)), ..Default::default() }];

    Self::find_many(mm, filter, None).await
  }

  /// 软删除作业
  pub async fn soft_delete_by_id(mm: &ModelManager, id: Uuid) -> Result<(), SqlError> {
    mm.dbx()
      .use_postgres(|dbx| async move {
        sqlx::query("UPDATE sched_job SET logical_deletion = NOW() WHERE id = $1")
          .bind(id)
          .execute(dbx.db())
          .await?;
        Ok(())
      })
      .await?;
    Ok(())
  }
}
