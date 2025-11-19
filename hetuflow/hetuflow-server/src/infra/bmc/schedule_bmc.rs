use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc},
  filter::{OpValInt32, OpValUuid},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use uuid::Uuid;

use hetuflow_core::types::{ScheduleKind, ScheduleStatus};

use hetuflow_core::models::{SchedSchedule, ScheduleFilter, ScheduleForCreate, ScheduleForUpdate};

/// ScheduleBmc 实现
pub struct ScheduleBmc;

impl DbBmc for ScheduleBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("sched_schedule").with_id_generated_by_db(false))
  }
}

generate_pg_bmc_common!(
  Bmc: ScheduleBmc,
  Entity: SchedSchedule,
  ForUpdate: ScheduleForUpdate,
  ForInsert: ScheduleForCreate,
);

generate_pg_bmc_filter!(
  Bmc: ScheduleBmc,
  Entity: SchedSchedule,
  Filter: ScheduleFilter,
);

impl ScheduleBmc {
  /// 查找可调度的 Schedule 实体（Cron 和 Time 类型且状态为 Enabled）
  pub async fn find_schedulable_entities(mm: &ModelManager) -> Result<Vec<SchedSchedule>, SqlError> {
    let filter = ScheduleFilter {
      schedule_kind: Some(OpValInt32::in_([ScheduleKind::Cron as i32, ScheduleKind::Interval as i32])),
      status: Some(OpValInt32::eq(ScheduleStatus::Enabled as i32)),
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
  pub async fn find_by_job_id(mm: &ModelManager, job_id: Uuid) -> Result<Vec<SchedSchedule>, SqlError> {
    let filter = ScheduleFilter { job_id: Some(OpValUuid::eq(job_id)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }
}
