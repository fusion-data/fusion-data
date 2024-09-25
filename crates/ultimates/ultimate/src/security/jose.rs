pub use josekit::{
  jwe::{alg::direct::DirectJweAlgorithm::Dir, JweHeader, ECDH_ES},
  jws::{JwsHeader, ES256, HS256},
  jwt::{self, JwtPayload},
  JoseError,
};

pub fn encrypt_jwe_ecdh_es(public_key: impl AsRef<[u8]>, payload: &JwtPayload) -> Result<String, JoseError> {
  let mut header = JweHeader::new();
  header.set_token_type("JWT");
  header.set_content_encryption("A128CBC-HS256");

  let encrypter = ECDH_ES.encrypter_from_pem(public_key)?;
  jwt::encode_with_encrypter(payload, &header, &encrypter)
}

pub fn decrypt_jwe_ecdh_es(
  private_key: impl AsRef<[u8]>,
  jwt: impl AsRef<[u8]>,
) -> Result<(JwtPayload, JweHeader), JoseError> {
  // Decrypting JWT
  let decrypter = ECDH_ES.decrypter_from_pem(private_key)?;
  jwt::decode_with_decrypter(jwt, &decrypter)
}

pub fn encrypt_jwe_dir(secret_key: impl AsRef<[u8]>, payload: &JwtPayload) -> Result<String, JoseError> {
  let mut header = JweHeader::new();
  header.set_token_type("JWT");
  header.set_content_encryption("A128CBC-HS256");

  let encrypter = Dir.encrypter_from_bytes(secret_key)?;
  jwt::encode_with_encrypter(payload, &header, &encrypter)
}

pub fn decrypt_jwe_dir(
  secret_key: impl AsRef<[u8]>,
  jwt: impl AsRef<[u8]>,
) -> Result<(JwtPayload, JweHeader), JoseError> {
  // Decrypting JWT
  let decrypter = Dir.decrypter_from_bytes(secret_key)?;
  jwt::decode_with_decrypter(jwt, &decrypter)
}

pub fn encode_jwt_es256(private_key: impl AsRef<[u8]>, payload: &JwtPayload) -> Result<String, JoseError> {
  let mut header = JwsHeader::new();
  header.set_token_type("JWT");

  // Signing JWT
  let signer = ES256.signer_from_pem(private_key)?;
  jwt::encode_with_signer(payload, &header, &signer)
}

pub fn decode_jwt_es256(
  public_key: impl AsRef<[u8]>,
  jwt: impl AsRef<[u8]>,
) -> Result<(JwtPayload, JwsHeader), JoseError> {
  // Verifing JWT
  let verifier = ES256.verifier_from_pem(public_key)?;
  jwt::decode_with_verifier(jwt, &verifier)
}

pub fn encode_jwt_hs256(secret_key: impl AsRef<[u8]>, payload: &JwtPayload) -> Result<String, JoseError> {
  let mut header = JwsHeader::new();
  header.set_token_type("JWT");

  // Signing JWT
  let signer = HS256.signer_from_bytes(secret_key)?;
  jwt::encode_with_signer(payload, &header, &signer)
}

pub fn decode_jwt_hs256(
  secret_key: impl AsRef<[u8]>,
  jwt: impl AsRef<[u8]>,
) -> Result<(JwtPayload, JwsHeader), JoseError> {
  // Verifing JWT
  let verifier = HS256.verifier_from_bytes(secret_key)?;
  jwt::decode_with_verifier(jwt, &verifier)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::configuration::{
    load_config,
    model::{KeyConf, SecurityConf},
  };
  use std::{
    sync::OnceLock,
    time::{Duration, SystemTime},
  };

  #[test]
  fn test_jwe_ecdh_es() -> anyhow::Result<()> {
    let (sc, expires_at) = helper();

    let mut jwt_payload = JwtPayload::new();
    jwt_payload.set_subject("subject");
    jwt_payload.set_expires_at(expires_at);

    // Encrypting JWT
    let jwt = encrypt_jwe_ecdh_es(sc.token().public_key(), &jwt_payload).unwrap();
    println!("Encrypting JWT with ECDH_ES signre is: {}", jwt);

    // Decrypting JWT
    let (payload, header) = decrypt_jwe_ecdh_es(sc.token().private_key(), jwt).unwrap();
    println!("Encrypting JWT with ECDH_ES JwsHeader is: {:?}", header);
    println!("Encrypting JWT with ECDH_ES JwtPayload is: {:?}", payload);

    assert_eq!(jwt_payload, payload);
    Ok(())
  }

  #[test]
  fn test_jwe_dir() -> anyhow::Result<()> {
    let secret_key = b"0123456789ABCDEF0123456789ABCDEF";
    let expires_at = SystemTime::now() + Duration::from_secs(60 * 60 * 24);

    let mut jwt_payload = JwtPayload::new();
    jwt_payload.set_subject("subject");
    jwt_payload.set_expires_at(&expires_at);

    // Encrypting JWT
    let jwt = encrypt_jwe_dir(secret_key, &jwt_payload).unwrap();
    println!("Encrypting JWT with DIR signre is: {}", jwt);

    // Decrypting JWT
    let (payload, header) = decrypt_jwe_dir(secret_key, jwt).unwrap();
    println!("Encrypting JWT with DIR JwsHeader is: {:?}", header);
    println!("Encrypting JWT with DIR JwtPayload is: {:?}", payload);

    assert_eq!(jwt_payload, payload);
    Ok(())
  }

  #[test]
  fn test_jwt_es256() -> anyhow::Result<()> {
    let (sc, expires_at) = helper();

    let mut jwt_payload = JwtPayload::new();
    jwt_payload.set_subject("subject");
    jwt_payload.set_expires_at(expires_at);

    // Signing JWT
    let jwt = encode_jwt_es256(sc.token().private_key(), &jwt_payload)?;
    println!("ES256 JWT signre is: {}", jwt);

    // Verifing JWT
    let (payload, header) = decode_jwt_es256(sc.token().public_key(), jwt)?;
    println!("ES256 JwsHeader is: {:?}", header);
    println!("ES256 JwtPayload is: {:?}", payload);

    assert_eq!(jwt_payload, payload);
    Ok(())
  }

  #[test]
  fn test_jwt_hs256() -> anyhow::Result<()> {
    let (sc, expires_at) = helper();

    let mut jwt_payload = JwtPayload::new();
    jwt_payload.set_subject("subject");
    jwt_payload.set_expires_at(expires_at);

    // Signing JWT
    println!("-- TOKEN JSON STR -- {}", serde_json::to_string(sc.token()).unwrap());
    let jwt = encode_jwt_hs256(sc.token().secret_key(), &jwt_payload)?;
    println!("HS256 JWT signre is: {}", jwt);

    // Verifing JWT
    let (payload, header) = decode_jwt_hs256(sc.token().secret_key(), jwt)?;
    println!("HS256 JwsHeader is: {:?}", header);
    println!("HS256 JwtPayload is: {:?}", payload);

    assert_eq!(jwt_payload, payload);
    Ok(())
  }

  // static PRIVATE_KEY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/pem/EC_P-256_private.pem");
  // static PUBLIC_KEY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/pem/EC_P-256_public.pem");
  static EXPIRES_AT: OnceLock<SystemTime> = OnceLock::new();
  static SECURITY_CONFIG: OnceLock<SecurityConf> = OnceLock::new();
  fn helper() -> (&'static SecurityConf, &'static SystemTime) {
    (
      SECURITY_CONFIG.get_or_init(|| load_config().unwrap().get::<SecurityConf>("ultimate.security").unwrap()),
      EXPIRES_AT.get_or_init(|| SystemTime::now().checked_add(Duration::from_secs(60 * 60 * 24 * 30)).unwrap()),
    )
  }
}
