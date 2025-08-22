use modelsql::{
  ModelManager, SqlError,
  base::DbBmc,
  filter::{OpValsInt32, OpValsUuid},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use uuid::Uuid;

use hetuflow_core::types::{ScheduleKind, ScheduleStatus};

use hetuflow_core::models::{ScheduleEntity, ScheduleFilter, ScheduleForCreate, ScheduleForUpdate};

/// ScheduleBmc 实现
pub struct ScheduleBmc;

impl DbBmc for ScheduleBmc {
  const TABLE: &str = "sched_schedule";
  const ID_GENERATED_BY_DB: bool = false;
}

generate_pg_bmc_common!(
  Bmc: ScheduleBmc,
  Entity: ScheduleEntity,
  ForUpdate: ScheduleForUpdate,
  ForInsert: ScheduleForCreate,
);

generate_pg_bmc_filter!(
  Bmc: ScheduleBmc,
  Entity: ScheduleEntity,
  Filter: ScheduleFilter,
);

impl ScheduleBmc {
  /// 查找可调度的 Schedule 实体（Cron 和 Time 类型且状态为 Enabled）
  pub async fn find_schedulable_entities(mm: &ModelManager) -> Result<Vec<ScheduleEntity>, SqlError> {
    let filter = ScheduleFilter {
      schedule_kind: Some(OpValsInt32::in_([ScheduleKind::Cron as i32, ScheduleKind::Interval as i32])),
      status: Some(OpValsInt32::eq(ScheduleStatus::Enabled as i32)),
      ..Default::default()
    };

    ScheduleBmc::find_many(mm, vec![filter], None).await
  }

  /// 更新调度状态
  pub async fn update_status_by_id(mm: &ModelManager, id: Uuid, status: ScheduleStatus) -> Result<(), SqlError> {
    let update = ScheduleForUpdate { status: Some(status), ..Default::default() };
    Self::update_by_id(mm, id, update).await
  }

  /// 根据作业ID查找调度
  pub async fn find_by_job_id(mm: &ModelManager, job_id: Uuid) -> Result<Vec<ScheduleEntity>, SqlError> {
    let filter = ScheduleFilter { job_id: Some(OpValsUuid::eq(job_id)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }
}
