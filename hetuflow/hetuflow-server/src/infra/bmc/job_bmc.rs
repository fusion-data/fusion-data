use modelsql::{
  ModelManager, SqlError,
  base::DbBmc,
  filter::{OpValsInt32, OpValsUuid},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use uuid::Uuid;

use hetuflow_core::types::JobStatus;

use hetuflow_core::models::{JobEntity, JobFilter, JobForCreate, JobForUpdate};

/// JobBmc 实现
pub struct JobBmc;

impl DbBmc for JobBmc {
  const TABLE: &str = "sched_job";
}

generate_pg_bmc_common!(
  Bmc: JobBmc,
  Entity: JobEntity,
  ForUpdate: JobForUpdate,
  ForInsert: JobForCreate,
);

generate_pg_bmc_filter!(
  Bmc: JobBmc,
  Entity: JobEntity,
  Filter: JobFilter,
);

impl JobBmc {
  /// 查找启用的作业
  pub async fn find_enabled_jobs(mm: &ModelManager) -> Result<Vec<JobEntity>, SqlError> {
    let filter = vec![JobFilter { status: Some(OpValsInt32::eq(JobStatus::Enabled as i32)), ..Default::default() }];

    Self::find_many(mm, filter, None).await
  }

  pub async fn find_enabled_by_id(mm: &ModelManager, id: Uuid) -> Result<Option<JobEntity>, SqlError> {
    let filter = vec![JobFilter {
      id: Some(OpValsUuid::eq(id)),
      status: Some(OpValsInt32::eq(JobStatus::Enabled as i32)),
      ..Default::default()
    }];

    Self::find_unique(mm, filter).await
  }

  /// 根据命名空间查找作业
  pub async fn find_jobs_by_namespace(mm: &ModelManager, namespace_id: Uuid) -> Result<Vec<JobEntity>, SqlError> {
    let filter = vec![JobFilter { namespace_id: Some(OpValsUuid::eq(namespace_id)), ..Default::default() }];

    Self::find_many(mm, filter, None).await
  }

  /// 软删除作业
  pub async fn soft_delete_by_id(mm: &ModelManager, id: Uuid) -> Result<(), SqlError> {
    mm.dbx()
      .use_postgres(|dbx| async move {
        sqlx::query("UPDATE sched_job SET deleted_at = NOW() WHERE id = $1")
          .bind(id)
          .execute(dbx.db())
          .await?;
        Ok(())
      })
      .await?;
    Ok(())
  }
}
