use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Utc};
use fusion_common::ctx::Ctx;

use crate::workflow::ExecutionId;

use super::Workflow;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
  execution_id: ExecutionId,
  workflow: Arc<Workflow>,
  ctx: Ctx,
  started_at: DateTime<FixedOffset>,
}

impl ExecutionContext {
  pub fn new(execution_id: ExecutionId, workflow: Arc<Workflow>, ctx: Ctx) -> Self {
    Self { execution_id, workflow, ctx, started_at: Utc::now().into() }
  }

  pub fn with_execution_id(mut self, execution_id: ExecutionId) -> Self {
    self.execution_id = execution_id;
    self
  }

  pub fn with_workflow(mut self, workflow: Arc<Workflow>) -> Self {
    self.workflow = workflow;
    self
  }

  pub fn with_ctx(mut self, ctx: Ctx) -> Self {
    self.ctx = ctx;
    self
  }

  pub fn with_started_at(mut self, started_at: DateTime<FixedOffset>) -> Self {
    self.started_at = started_at;
    self
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

  pub fn started_at(&self) -> &DateTime<FixedOffset> {
    &self.started_at
  }
}
