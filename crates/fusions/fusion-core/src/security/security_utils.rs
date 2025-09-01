use std::time::SystemTime;

use fusion_corelib::ctx::CtxPayload;
use josekit::{jwe::JweHeader, jwt::JwtPayload};

use crate::configuration::KeyConf;

use super::{
  Error,
  jose::{decrypt_jwe_dir, encrypt_jwe_dir},
};

pub struct SecurityUtils;

impl SecurityUtils {
  pub fn encrypt_jwt(key_conf: &dyn KeyConf, payload: CtxPayload) -> Result<String, Error> {
    let payload = JwtPayload::from_map(payload.into_inner())?;
    Self::encrypt_jwt_payload(key_conf, payload)
  }

  pub fn encrypt_jwt_payload(key_conf: &dyn KeyConf, mut payload: JwtPayload) -> Result<String, Error> {
    if payload.expires_at().is_none() {
      let expires_at = key_conf.expires_at().into();
      payload.set_expires_at(&expires_at);
    }

    encrypt_jwe_dir(key_conf.secret_key(), &payload).map_err(Error::JoseError)
  }

  pub fn decrypt_jwt(key_conf: &dyn KeyConf, token: &str) -> Result<(CtxPayload, JweHeader), Error> {
    let (payload, header) = Self::decrypt_jwt_payload(key_conf, token)?;
    let payload = CtxPayload::new(payload.into());
    Ok((payload, header))
  }

  pub fn decrypt_jwt_payload(key_conf: &dyn KeyConf, token: &str) -> Result<(JwtPayload, JweHeader), Error> {
    let (payload, header) = decrypt_jwe_dir(key_conf.secret_key(), token)?;
    if let Some(expires_at) = payload.expires_at()
      && expires_at < SystemTime::now()
    {
      return Err(Error::TokenExpired);
    }
    Ok((payload, header))
  }
}
#[cfg(test)]
mod tests {
  use std::time::{Duration, SystemTime};

  use fusion_corelib::ctx::CtxPayload;

  use crate::configuration::TokenConf;

  use super::*;

  fn make_key_conf() -> TokenConf {
    TokenConf {
      secret_key: b"0123456789ABCDEF0123456789ABCDEF".to_vec(),
      expires_in: 7200,
      public_key: br#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEOTv4YquENmDfXoSN0TQiOqmgR1Px
UDTicuyW06VcX/XOkXp/6vmIIBFUXVWREJmQy7EIhNXM1qCy7Hs6SK9y7A==
-----END PUBLIC KEY-----"#
        .to_vec(),
      private_key: br#"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgbMlaUVhOz9IHvlxT
4i7Wm6cmubzzGZr/PNNME25ZVNuhRANCAAQ5O/hiq4Q2YN9ehI3RNCI6qaBHU/FQ
NOJy7JbTpVxf9c6Ren/q+YggEVRdVZEQmZDLsQiE1czWoLLsezpIr3Ls
-----END PRIVATE KEY-----"#
        .to_vec(),
    }
  }

  #[test]
  fn test_encrypt_and_decrypt_jwt() {
    let key_conf = make_key_conf();
    let mut payload = CtxPayload::default();
    payload.set_subject("test_user");
    payload.set_exp(
      (SystemTime::now() + Duration::from_secs(600))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64,
    );

    let token = SecurityUtils::encrypt_jwt(&key_conf, payload.clone()).expect("encrypt_jwt failed");
    let (decrypted_payload, _header) = SecurityUtils::decrypt_jwt(&key_conf, &token).expect("decrypt_jwt failed");

    assert_eq!(decrypted_payload.get_subject(), payload.get_subject());
    assert!(decrypted_payload.get_exp().is_some());
  }

  #[test]
  fn test_token_expired() {
    let mut key_conf = make_key_conf();
    key_conf.expires_in = -10;
    let mut payload = CtxPayload::default();
    payload.set_subject("expired_user");
    payload.set_exp(
      (SystemTime::now() - Duration::from_secs(20))
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64,
    );

    let token = SecurityUtils::encrypt_jwt(&key_conf, payload).expect("encrypt_jwt failed");
    let result = SecurityUtils::decrypt_jwt(&key_conf, &token);

    assert!(matches!(result, Err(Error::TokenExpired)));
  }
}
