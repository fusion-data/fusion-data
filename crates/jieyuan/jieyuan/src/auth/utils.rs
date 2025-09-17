use fusion_common::ctx::CtxPayload;
use fusion_core::{DataError, Result, configuration::SecuritySetting, security::SecurityUtils};

pub fn make_token(sc: &SecuritySetting, uid: i64) -> Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid.to_string());

  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}
