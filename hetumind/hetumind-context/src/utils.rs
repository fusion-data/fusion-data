use http::request::Parts;
use log::error;
use modelsql::ModelManager;
use ultimate_common::ctx::{Ctx, CtxPayload};
use ultimate_core::{
  DataError,
  application::Application,
  configuration::{KeyConf, PwdConf},
  security::SecurityUtils,
};
use ultimate_web::WebError;

use crate::ctx::CtxW;

/// 生成 password token
pub fn make_token(sub: impl Into<String>, pwd_conf: &PwdConf) -> Result<String, DataError> {
  let mut payload = CtxPayload::default();
  payload.set_subject(sub);
  payload.set_exp(pwd_conf.expires_at().timestamp());

  let token = SecurityUtils::encrypt_jwt(pwd_conf, payload).map_err(|e| {
    let msg = format!("Generate token failed: {}", e);
    error!("{}", msg);
    DataError::unauthorized(msg)
  })?;
  Ok(token)
}

pub fn new_ctx_w_from_parts(parts: &Parts, state: &Application) -> Result<CtxW, WebError> {
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "Unauthorized"))?;
  let mm = state.component::<ModelManager>();
  let mm = mm.with_ctx(ctx.clone());
  let ctx_w = CtxW::new(mm, Default::default());
  Ok(ctx_w)
}
