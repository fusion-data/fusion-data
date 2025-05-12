use std::sync::Arc;

use modelsql::ModelManager;
use tonic::{metadata::MetadataMap, Extensions, Status};
use tracing::error;
use ultimate_common::{
  ctx::{Ctx, CtxPayload},
  time::now_utc,
};
use ultimate_core::application::Application;
use ultimate_grpc::utils::extract_payload_from_metadata;

static X_APP_VERSION: &str = "X-APP-VARSION";
static X_DEVICE_ID: &str = "X-DEVICE-ID";

#[derive(Clone)]
pub struct CtxW {
  // app: Application,
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

impl TryFrom<&MetadataMap> for CtxW {
  type Error = Status;
  fn try_from(metadata: &MetadataMap) -> core::result::Result<Self, Status> {
    let app = Application::global();

    let sc = app.ultimate_config().security();
    let req_time = now_utc();

    let payload = extract_payload_from_metadata(sc, metadata)?;
    let req_meta = RequestMetadata::from(metadata);
    let request_id = metadata.get("request_id").and_then(|v| v.to_str().ok().map(|s| s.to_string()));

    let ctx = Ctx::new(payload, Some(req_time), request_id);
    let mm = app
      .get_component::<ModelManager>()
      .map_err(|e| {
        error!("ModelManager not found, error: {:?}", e);
        Status::internal("ModelManager not found.")
      })?
      .with_ctx(ctx);
    Ok(CtxW::new(mm, Arc::new(req_meta)))
  }
}

impl<'a> TryFrom<&'a Extensions> for &'a CtxW {
  type Error = Status;

  fn try_from(extensions: &'a Extensions) -> Result<&'a CtxW, Status> {
    extensions.get().ok_or_else(|| Status::unauthenticated("未经身份验证"))
  }
}

#[derive(Clone, Default)]
pub struct RequestMetadata {
  app_ver: String,
  dev_id: String,
}

impl RequestMetadata {
  pub fn app_ver(&self) -> &str {
    self.app_ver.as_str()
  }

  pub fn dev_id(&self) -> &str {
    self.dev_id.as_str()
  }
}

impl From<&MetadataMap> for RequestMetadata {
  fn from(metadata: &MetadataMap) -> Self {
    let app_ver = metadata.get(X_APP_VERSION).map(|v| v.to_str().unwrap_or("").to_string()).unwrap_or_default();
    let dev_id = metadata.get(X_DEVICE_ID).map(|v| v.to_str().unwrap_or("").to_string()).unwrap_or_default();
    Self { app_ver, dev_id }
  }
}
