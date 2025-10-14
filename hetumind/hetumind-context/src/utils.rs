use fusion_common::ctx::{Ctx, CtxPayload};
use fusion_core::{
  DataError,
  application::Application,
  configuration::{KeyConf, PwdConf},
  security::SecurityUtils,
};
use fusion_web::WebError;
use fusionsql::ModelManager;
use http::request::Parts;
use log::error;

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

pub fn get_mm_from_parts(parts: &Parts, state: &Application) -> Result<ModelManager, WebError> {
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "Unauthorized"))?;
  let mm = state.component::<ModelManager>().with_ctx(ctx.clone());
  Ok(mm)
}
