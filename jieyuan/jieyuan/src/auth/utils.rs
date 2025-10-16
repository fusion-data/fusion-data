use fusion_common::ctx::CtxPayload;
use fusion_core::{DataError, Result, configuration::SecuritySetting, security::SecurityUtils};

pub fn make_token(sc: &SecuritySetting, uid: i64) -> Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid.to_string());

  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}

/// 生成包含租户ID的令牌
pub fn make_token_with_tenant(sc: &SecuritySetting, uid: i64, tenant_id: i64) -> Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid.to_string());
  payload.set_i64("tenant_id", tenant_id);

  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}

/// 验证 token 并提取用户 ID
pub fn validate_token(token: &str) -> Result<i64> {
  let config = fusion_core::application::Application::global().fusion_config();
  let (payload, _header) = SecurityUtils::decrypt_jwt(config.security().pwd(), token)
    .map_err(|_e| DataError::unauthorized("Invalid token"))?;

  let uid_str = payload.get_str("sub").ok_or_else(|| DataError::unauthorized("Missing subject in token"))?;
  uid_str.parse::<i64>().map_err(|_e| DataError::unauthorized("Invalid user ID in token"))
}

/// 验证 token 并提取用户 ID 和租户 ID
pub fn validate_token_with_tenant(token: &str) -> Result<(i64, i64)> {
  let config = fusion_core::application::Application::global().fusion_config();
  let (payload, _header) = SecurityUtils::decrypt_jwt(config.security().pwd(), token)
    .map_err(|_e| DataError::unauthorized("Invalid token"))?;

  let uid_str = payload.get_str("sub").ok_or_else(|| DataError::unauthorized("Missing subject in token"))?;
  let uid = uid_str.parse::<i64>().map_err(|_e| DataError::unauthorized("Invalid user ID in token"))?;

  let tenant_id = payload.get_i64("tenant_id").ok_or_else(|| DataError::unauthorized("Missing tenant_id in token"))?;

  Ok((uid, tenant_id))
}
