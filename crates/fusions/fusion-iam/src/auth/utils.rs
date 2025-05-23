use ultimate_common::ctx::CtxPayload;
use ultimate_core::{DataError, Result, configuration::SecurityConfig, security::SecurityUtils};

pub fn make_token(sc: &SecurityConfig, uid: i64) -> Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid.to_string());

  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}
