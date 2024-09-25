use josekit::{jwe::JweHeader, jwt::JwtPayload, JoseError};

use crate::configuration::model::KeyConf;

use super::jose::{decrypt_jwe_dir, encrypt_jwe_dir};

pub struct SecurityUtils;

impl SecurityUtils {
  pub fn encrypt_jwt(key_conf: &dyn KeyConf, mut payload: JwtPayload) -> Result<String, JoseError> {
    if payload.expires_at().is_none() {
      let expires_at = key_conf.expires_at().into();
      payload.set_expires_at(&expires_at);
    }
    encrypt_jwe_dir(key_conf.secret_key(), &payload)
  }

  pub fn decrypt_jwt(key_conf: &dyn KeyConf, token: &str) -> Result<(JwtPayload, JweHeader), JoseError> {
    decrypt_jwe_dir(key_conf.secret_key(), token)
  }
}
