use axum::extract::FromRequestParts;
use fusion_core::{DataError, application::Application};
use fusion_web::WebError;
use hetumind_context::{ctx::CtxW, utils::new_ctx_w_from_parts};
use hetumind_core::workflow::{Execution, ExecutionData, ExecutionForQuery, ExecutionForUpdate, ExecutionId};
use http::request::Parts;
use modelsql::page::PageResult;

use crate::domain::workflow::ExecutionBmc;

pub struct ExecutionSvc {
  pub ctx: CtxW,
}

impl ExecutionSvc {
  pub async fn cancel_execution(&self, execution_id: ExecutionId) -> Result<(), DataError> {
    todo!()
  }

  pub async fn retry_execution(&self, execution_id: ExecutionId) -> Result<(), DataError> {
    todo!()
  }

  pub async fn logs(&self, execution_id: ExecutionId) -> Result<Vec<ExecutionData>, DataError> {
    todo!()
  }

  pub async fn query_executions(&self, input: ExecutionForQuery) -> Result<PageResult<Execution>, DataError> {
    todo!()
  }

  pub async fn find_execution_by_id(&self, execution_id: ExecutionId) -> Result<Execution, DataError> {
    let entity = ExecutionBmc::find_by_id(self.ctx.mm(), execution_id).await?;
    // let execution = Execution::try_from(entity)?;
    todo!()
  }

  pub async fn create_execution(&self, input: Execution) -> Result<Execution, DataError> {
    todo!()
  }

  pub async fn update_execution(
    &self,
    execution_id: ExecutionId,
    input: ExecutionForUpdate,
  ) -> Result<Execution, DataError> {
    todo!()
  }

  pub async fn delete_execution(&self, execution_id: ExecutionId) -> Result<(), DataError> {
    todo!()
  }
}

impl ExecutionSvc {
  pub fn new(ctx: CtxW) -> Self {
    Self { ctx }
  }
}

impl FromRequestParts<Application> for ExecutionSvc {
  type Rejection = WebError;

  async fn from_request_parts(parts: &mut Parts, state: &Application) -> Result<Self, Self::Rejection> {
    new_ctx_w_from_parts(parts, state).map(Self::new)
  }
}
