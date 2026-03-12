use hetu_common::{DataError, ctx::CtxPayload};
use hetu_core::{configuration::SecuritySetting, security::SecurityUtils};

pub fn make_token(sc: &SecuritySetting, payload: CtxPayload) -> Result<String, DataError> {
  let token =
    SecurityUtils::encrypt_jwt(sc.pwd(), payload).map_err(|_e| DataError::unauthorized("Failed generate token"))?;
  Ok(token)
}

pub fn make_token_by_user_id(sc: &SecuritySetting, uid: impl Into<String>) -> Result<String, DataError> {
  let mut payload = CtxPayload::default();
  payload.set_subject(uid);
  make_token(sc, payload)
}
