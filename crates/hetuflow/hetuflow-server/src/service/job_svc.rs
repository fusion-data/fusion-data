use fusion_core::DataError;
use modelsql::{ModelManager, page::PageResult};
use uuid::Uuid;

use hetuflow_core::models::{JobForCreate, JobForQuery, JobForUpdate, SchedJob};
use hetuflow_core::types::JobStatus;

use crate::infra::bmc::JobBmc;

pub struct JobSvc {
  pub(crate) mm: ModelManager,
}

impl JobSvc {
  pub async fn query(&self, input: JobForQuery) -> Result<PageResult<SchedJob>, DataError> {
    JobBmc::page(&self.mm, vec![input.filter], input.page).await.map_err(DataError::from)
  }

  pub async fn create(&self, mut input: JobForCreate) -> Result<Uuid, DataError> {
    let id = if let Some(id) = input.id {
      id
    } else {
      let id = Uuid::now_v7();
      input.id = Some(id);
      id
    };
    JobBmc::insert(&self.mm, input).await?;
    Ok(id)
  }

  /// 根据 ID 获取任务
  pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<SchedJob>, DataError> {
    JobBmc::get_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  /// 根据 ID 更新任务
  pub async fn update_by_id(&self, id: &Uuid, input: JobForUpdate) -> Result<(), DataError> {
    JobBmc::update_by_id(&self.mm, id, input).await.map_err(DataError::from)
  }

  /// 根据 ID 删除作业
  pub async fn delete_by_id(&self, id: &Uuid) -> Result<(), DataError> {
    JobBmc::delete_by_id(&self.mm, *id).await.map_err(DataError::from)
  }

  /// 更新任务状态
  pub async fn update_status(&self, id: &Uuid, status: JobStatus) -> Result<(), DataError> {
    let update = JobForUpdate { status: Some(status), ..Default::default() };
    self.update_by_id(id, update).await
  }
}
