use std::sync::Arc;

use fusion_core::{DataError, application::Application};
use fusion_common::ctx::Ctx;
use fusion_web::WebError;
use fusionsql::ModelManager;
use http::request::Parts;

use crate::meta::RequestMetadata;

#[derive(Clone)]
pub struct CtxW {
  mm: ModelManager,
  req_meta: Arc<RequestMetadata>,
}

impl CtxW {
  pub fn new(mm: ModelManager, req_meta: Arc<RequestMetadata>) -> Self {
    Self { mm, req_meta }
  }

  pub fn new_super_admin(mm: ModelManager) -> Self {
    Self::new(mm.with_ctx(Ctx::new_super_admin()), Default::default())
  }

  // pub fn ctx(&self) -> &Ctx {
  //   self.mm().ctx_ref()
  //   &self.ctx
  // }

  pub fn app(&self) -> Application {
    Application::global()
  }

  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }

  pub fn ctx(&self) -> Result<&Ctx, DataError> {
    self.mm.ctx_ref().map_err(DataError::from)
  }

  pub fn req_meta(&self) -> &Arc<RequestMetadata> {
    &self.req_meta
  }

  pub fn into_tx_mm_ctx(self) -> CtxW {
    let mm = self.mm.get_txn_clone();
    self.with_mm(mm)
  }

  pub fn with_mm(self, mm: ModelManager) -> Self {
    Self { mm, ..self }
  }
}

impl<'a> TryFrom<&'a Parts> for &'a CtxW {
  type Error = WebError;

  fn try_from(parts: &'a Parts) -> Result<&'a CtxW, WebError> {
    parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "Unauthorized"))
  }
}
