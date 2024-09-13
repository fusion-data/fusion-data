use std::sync::Arc;

use derive_getters::Getters;
use tonic::{metadata::MetadataMap, Extensions, Status};
use ultimate::ctx::Ctx;
use ultimate_common::time::now_utc;
use ultimate_db::ModelManager;
use ultimate_grpc::utils::extract_jwt_payload_from_metadata;

use crate::app::{get_app_state, AppState};

static X_APP_VERSION: &str = "X-APP-VARSION";
static X_DEVICE_ID: &str = "X-DEVICE-ID";

#[derive(Clone, Getters)]
pub struct CtxW {
  ctx: Ctx,
  mm: ModelManager,
  req_meta: Arc<RequestMetadata>,
}

impl CtxW {
  pub fn new(state: &AppState, ctx: Ctx, req_meta: Arc<RequestMetadata>) -> Self {
    let mm = state.mm().clone().with_ctx(ctx.clone());
    Self { ctx, mm, req_meta }
  }
}

impl TryFrom<&MetadataMap> for CtxW {
  type Error = Status;
  fn try_from(metadata: &MetadataMap) -> core::result::Result<Self, Status> {
    let app = get_app_state();
    let sc = app.ultimate_config().security();
    let req_time = now_utc();

    let payload = extract_jwt_payload_from_metadata(sc, metadata)?;
    let req_meta = RequestMetadata::from(metadata);

    let ctx = Ctx::try_from_jwt_payload(&payload, Some(req_time))?;
    Ok(CtxW::new(app, ctx, Arc::new(req_meta)))
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
