use ultimate_common::ctx::CtxPayload;
use ultimate_core::{DataError, configuration::SecuritySetting, security::SecurityUtils};

pub fn make_token(sc: &SecuritySetting, payload: CtxPayload) -> ultimate_core::Result<String> {
  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}

pub fn make_token_by_user_id(sc: &SecuritySetting, uid: impl Into<String>) -> ultimate_core::Result<String> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid);
  make_token(sc, payload)
}
