use fusion_common::ctx::{Ctx, CtxPayload};
use fusion_common::time::{Duration, now_utc};
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

/// 生成刷新令牌（长期有效）
pub fn make_refresh_token(sub: impl Into<String>, pwd_conf: &PwdConf) -> Result<String, DataError> {
  let mut payload = CtxPayload::default();
  payload.set_subject(sub);
  // 刷新令牌有效期设置为30天
  let refresh_expires_utc = now_utc() + Duration::days(30);
  payload.set_exp(refresh_expires_utc.timestamp());

  let token = SecurityUtils::encrypt_jwt(pwd_conf, payload).map_err(|e| {
    let msg = format!("Generate refresh token failed: {}", e);
    error!("{}", msg);
    DataError::unauthorized(msg)
  })?;
  Ok(token)
}

/// 验证并解析令牌
pub fn verify_token(token: &str, pwd_conf: &PwdConf) -> Result<CtxPayload, DataError> {
  let (payload, _header) = SecurityUtils::decrypt_jwt(pwd_conf, token).map_err(|e| {
    let msg = format!("Invalid token: {}", e);
    error!("{}", msg);
    DataError::unauthorized(msg)
  })?;
  Ok(payload)
}

pub fn get_mm_from_parts(parts: &Parts, state: &Application) -> Result<ModelManager, WebError> {
  let ctx: &Ctx = parts.extensions.get().ok_or_else(|| WebError::new_with_code(401, "Unauthorized"))?;
  let mm = state.component::<ModelManager>().with_ctx(ctx.clone());
  Ok(mm)
}
