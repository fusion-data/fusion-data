use fusion_core::DataError;
use modelsql::{ModelManager, page::PageResult};
use uuid::Uuid;

use hetuflow_core::models::{SchedSchedule, ScheduleForCreate, ScheduleForQuery, ScheduleForUpdate};

use crate::infra::bmc::ScheduleBmc;

pub struct ScheduleSvc {
  pub(crate) mm: ModelManager,
}

impl ScheduleSvc {
  pub async fn query(&self, input: ScheduleForQuery) -> Result<PageResult<SchedSchedule>, DataError> {
    ScheduleBmc::page(&self.mm, vec![input.filter], input.page).await.map_err(DataError::from)
  }

  pub async fn create(&self, input: ScheduleForCreate) -> Result<Uuid, DataError> {
    let id = input.id;
    ScheduleBmc::insert(&self.mm, input).await?;
    Ok(id)
  }

  /// 根据 ID 获取调度计划
  pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<SchedSchedule>, DataError> {
    ScheduleBmc::get_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  /// 根据 ID 更新调度计划
  pub async fn update_by_id(&self, id: &Uuid, input: ScheduleForUpdate) -> Result<(), DataError> {
    ScheduleBmc::update_by_id(&self.mm, id, input).await.map_err(DataError::from)
  }

  /// 根据 ID 删除调度计划
  pub async fn delete_by_id(&self, id: &Uuid) -> Result<(), DataError> {
    ScheduleBmc::delete_by_id(&self.mm, *id).await.map_err(DataError::from)
  }

  /// 查找可调度的调度计划
  pub async fn find_schedulable(&self) -> Result<Vec<SchedSchedule>, DataError> {
    ScheduleBmc::find_schedulable_entities(&self.mm).await.map_err(DataError::from)
  }
}
