use std::sync::Arc;

use fusion_common::ctx::Ctx;
use fusion_common::time::{OffsetDateTime, now};

use crate::workflow::ExecutionId;

use super::Workflow;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
  execution_id: ExecutionId,
  workflow: Arc<Workflow>,
  ctx: Ctx,
  started_at: OffsetDateTime,
}

impl ExecutionContext {
  pub fn new(execution_id: ExecutionId, workflow: Arc<Workflow>, ctx: Ctx) -> Self {
    Self { execution_id, workflow, ctx, started_at: now() }
  }

  pub fn ctx(&self) -> &Ctx {
    &self.ctx
  }

  pub fn execution_id(&self) -> &ExecutionId {
    &self.execution_id
  }

  pub fn workflow(&self) -> Arc<Workflow> {
    self.workflow.clone()
  }

  pub fn started_at(&self) -> &OffsetDateTime {
    &self.started_at
  }
}
