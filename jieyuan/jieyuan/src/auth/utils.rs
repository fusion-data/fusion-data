use fusion_common::ctx::CtxPayload;
use fusion_core::{DataError, Result, configuration::SecuritySetting, security::SecurityUtils};

pub fn make_token(sc: &SecuritySetting, uid: i64) -> Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid.to_string());

  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}

/// 生成包含租户ID和令牌序列的令牌
pub fn make_token_with_tenant(sc: &SecuritySetting, uid: i64, tenant_id: i64, token_seq: i32) -> Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid.to_string());
  payload.set_i64("tenant_id", tenant_id);
  payload.set_i32("token_seq", token_seq);

  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}

/// 验证令牌序列是否有效（与数据库中的当前序列比对）
pub async fn validate_token_seq_against_db(
  user_id: i64,
  tenant_id: i64,
  token_seq: i32,
  mm: &fusionsql::ModelManager,
) -> Result<()> {
  use crate::user::UserCredentialBmc;

  let user_credential = UserCredentialBmc::get_by_id_for_update(mm, user_id, tenant_id)
    .await?
    .ok_or_else(|| DataError::unauthorized("User not found in specified tenant"))?;

  if user_credential.token_seq != token_seq {
    return Err(DataError::unauthorized("Token has been invalidated due to password change"));
  }

  Ok(())
}

/// 验证 token 并提取用户 ID
pub fn validate_token(token: &str) -> Result<i64> {
  let config = fusion_core::application::Application::global().fusion_setting();
  let (payload, _header) = SecurityUtils::decrypt_jwt(config.security().pwd(), token)
    .map_err(|_e| DataError::unauthorized("Invalid token"))?;

  let uid_str = payload.get_str("sub").ok_or_else(|| DataError::unauthorized("Missing subject in token"))?;
  uid_str.parse::<i64>().map_err(|_e| DataError::unauthorized("Invalid user ID in token"))
}

/// 验证 token 并提取用户 ID 和租户 ID
pub fn validate_token_with_tenant(token: &str) -> Result<(i64, i64)> {
  let config = fusion_core::application::Application::global().fusion_setting();
  let (payload, _header) = SecurityUtils::decrypt_jwt(config.security().pwd(), token)
    .map_err(|_e| DataError::unauthorized("Invalid token"))?;

  let uid_str = payload.get_str("sub").ok_or_else(|| DataError::unauthorized("Missing subject in token"))?;
  let uid = uid_str.parse::<i64>().map_err(|_e| DataError::unauthorized("Invalid user ID in token"))?;

  let tenant_id = payload.get_i64("tenant_id").ok_or_else(|| DataError::unauthorized("Missing tenant_id in token"))?;

  Ok((uid, tenant_id))
}

/// 验证 token 并提取用户 ID、租户 ID 和令牌序列
pub fn validate_token_with_tenant_and_seq(token: &str) -> Result<(i64, i64, i32)> {
  let config = fusion_core::application::Application::global().fusion_setting();
  let (payload, _header) = SecurityUtils::decrypt_jwt(config.security().pwd(), token)
    .map_err(|_e| DataError::unauthorized("Invalid token"))?;

  let uid_str = payload.get_str("sub").ok_or_else(|| DataError::unauthorized("Missing subject in token"))?;
  let uid = uid_str.parse::<i64>().map_err(|_e| DataError::unauthorized("Invalid user ID in token"))?;

  let tenant_id = payload.get_i64("tenant_id").ok_or_else(|| DataError::unauthorized("Missing tenant_id in token"))?;
  let token_seq = payload.get_i32("token_seq").unwrap_or(0); // 默认为 0 兼容旧令牌

  Ok((uid, tenant_id, token_seq))
}

/// 从 Http Request Parts 中获取并验证令牌序列的 Ctx
pub async fn extract_ctx_with_token_seq_validation(
  parts: &axum::http::request::Parts,
  mm: &fusionsql::ModelManager,
) -> Result<fusion_common::ctx::Ctx> {
  use fusion_core::application::Application;
  use std::time::SystemTime;

  let app_config = Application::global().fusion_setting();
  let security_config = app_config.security();

  // 获取令牌
  let token = if let Some(bearer) = parts
    .headers
    .get("authorization")
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.strip_prefix("Bearer "))
  {
    bearer.to_string()
  } else if let Some(query) = parts.uri.query() {
    format!("?{}", query)
      .split('&')
      .find(|s| s.starts_with("access_token="))
      .and_then(|s| s.strip_prefix("access_token="))
      .unwrap_or("")
      .to_string()
  } else {
    return Err(DataError::unauthorized("Missing token"));
  };

  if token.is_empty() {
    return Err(DataError::unauthorized("Missing token"));
  }

  // 验证令牌并获取信息
  let (user_id, tenant_id, token_seq) = validate_token_with_tenant_and_seq(&token)?;

  // 验证令牌序列是否有效
  validate_token_seq_against_db(user_id, tenant_id, token_seq, mm).await?;

  // 创建标准 Ctx
  let req_time = SystemTime::now();
  let (payload, _) = SecurityUtils::decrypt_jwt(security_config.pwd(), &token)
    .map_err(|_e| DataError::unauthorized("Failed decode jwt"))?;

  let ctx = fusion_common::ctx::Ctx::try_new(payload, Some(req_time.into()), None)
    .map_err(|e| DataError::unauthorized(e.to_string()))?;

  Ok(ctx)
}
