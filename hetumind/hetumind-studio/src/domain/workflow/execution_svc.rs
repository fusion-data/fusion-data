use axum::extract::FromRequestParts;
use hetumind_context::utils::get_mm_from_parts;
use hetumind_core::workflow::{Execution, ExecutionData, ExecutionForQuery, ExecutionForUpdate, ExecutionId};
use http::request::Parts;
use ultimates::core::{DataError, application::Application};
use ultimates::web::WebError;
use ultimatesql::{ModelManager, page::PageResult};

use crate::domain::workflow::ExecutionBmc;

pub struct ExecutionSvc {
  pub mm: ModelManager,
}

impl ExecutionSvc {
  pub async fn cancel_execution(&self, _execution_id: ExecutionId) -> Result<(), DataError> {
    todo!()
  }

  pub async fn retry_execution(&self, _execution_id: ExecutionId) -> Result<(), DataError> {
    todo!()
  }

  pub async fn logs(&self, _execution_id: ExecutionId) -> Result<Vec<ExecutionData>, DataError> {
    todo!()
  }

  pub async fn query_executions(&self, _input: ExecutionForQuery) -> Result<PageResult<Execution>, DataError> {
    todo!()
  }

  pub async fn find_execution_by_id(&self, execution_id: ExecutionId) -> Result<Execution, DataError> {
    let _entity = ExecutionBmc::find_by_id(&self.mm, execution_id).await?;
    // let execution = Execution::try_from(entity)?;
    todo!()
  }

  pub async fn create_execution(&self, _input: Execution) -> Result<Execution, DataError> {
    todo!()
  }

  pub async fn update_execution(
    &self,
    _execution_id: ExecutionId,
    _input: ExecutionForUpdate,
  ) -> Result<Execution, DataError> {
    todo!()
  }

  pub async fn delete_execution(&self, _execution_id: ExecutionId) -> Result<(), DataError> {
    todo!()
  }
}

impl ExecutionSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }
}

impl FromRequestParts<Application> for ExecutionSvc {
  type Rejection = WebError;

  async fn from_request_parts(parts: &mut Parts, state: &Application) -> Result<Self, Self::Rejection> {
    get_mm_from_parts(parts, state).map(Self::new)
  }
}
