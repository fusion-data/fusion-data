use josekit::{jwe::JweHeader, jwt::JwtPayload, JoseError};
use ultimate_common::ctx::CtxPayload;

use crate::configuration::KeyConf;

use super::jose::{decrypt_jwe_dir, encrypt_jwe_dir};

pub struct SecurityUtils;

impl SecurityUtils {
  pub fn encrypt_jwt(key_conf: &dyn KeyConf, mut payload: CtxPayload) -> Result<String, JoseError> {
    if payload.get_expires_at().is_none() {
      let expires_at = key_conf.expires_at();
      payload.set_expires_at(expires_at);
    }
    let payload = JwtPayload::from_map(payload.into_inner())?;
    encrypt_jwe_dir(key_conf.secret_key(), &payload)
  }

  pub fn decrypt_jwt(key_conf: &dyn KeyConf, token: &str) -> Result<(CtxPayload, JweHeader), JoseError> {
    let (payload, header) = decrypt_jwe_dir(key_conf.secret_key(), token)?;
    let payload = CtxPayload::new(payload.into());
    Ok((payload, header))
  }
}
