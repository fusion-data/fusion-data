use fusion_core::DataError;
use hetuflow_core::models::{SchedServer, ServerForQuery, ServerForUpdate};
use modelsql::{ModelManager, page::PageResult};

use crate::infra::bmc::ServerBmc;

pub struct ServerSvc {
  mm: ModelManager,
}

impl ServerSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn get_by_id(&self, id: &str) -> Result<Option<SchedServer>, DataError> {
    ServerBmc::get_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  pub async fn update_by_id(&self, id: &str, data: ServerForUpdate) -> Result<(), DataError> {
    ServerBmc::update_by_id(&self.mm, id, data).await.map_err(DataError::from)
  }

  pub async fn delete_by_id(&self, id: &str) -> Result<(), DataError> {
    ServerBmc::delete_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  pub async fn page(&self, query: ServerForQuery) -> Result<PageResult<SchedServer>, DataError> {
    ServerBmc::page(&self.mm, vec![query.filter], query.page).await.map_err(DataError::from)
  }
}
