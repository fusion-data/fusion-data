use std::sync::OnceLock;

use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use regex::Regex;
use tracing::{error, trace};

use super::{Error, Result};

const CUR_PWD_VERSION: u16 = 1;

pub async fn generate_pwd(password: &str) -> Result<String> {
  let hash = try_to_hash(password).await?;
  Ok(format!("#{}#{}", CUR_PWD_VERSION, hash))
}

pub async fn verify_pwd(password: &str, hashed_pwd: &str) -> Result<u16> {
  let (version, hash) = split_pwd_version(hashed_pwd);
  if verify(password.as_bytes(), hash).await? {
    Ok(version)
  } else {
    Err(Error::InvalidPassword)
  }
}

pub(crate) async fn try_to_hash(password: &str) -> Result<String> {
  let salt = SaltString::generate(rand::thread_rng());
  let hash = Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .map_err(|e| {
      error!("Failed to hash password: {}", e.to_string());
      Error::FailedToHashPassword
    })?
    .to_string();
  Ok(hash)
}

pub(crate) async fn verify(password: &[u8], hash: &str) -> Result<bool> {
  let hash = PasswordHash::new(hash).map_err(|e| {
    error!("BUG: password hash invalid: {}", e.to_string());
    Error::InvalidFormat
  })?;

  let res = Argon2::default().verify_password(password, &hash);

  match res {
    Ok(()) => Ok(true),
    Err(password_hash::Error::Password) => Ok(false),
    Err(e) => {
      error!("Failed to verify password: {}", e.to_string());
      Err(Error::FailedToVerifyPassword)
    }
  }
}

static SPLIT_PWD_RE: OnceLock<Regex> = OnceLock::new();

fn split_pwd_version(pwd: &str) -> (u16, &str) {
  let re = SPLIT_PWD_RE.get_or_init(|| Regex::new(r"^#(?<version>\d+)#").unwrap());
  if let Some(caps) = re.captures(pwd) {
    trace!("The version of pwd is {:?}", caps);
    let version = caps.name("version").unwrap();
    trace!(
      "start:{}, end:{}, len:{}, range:{:?}, str:{}",
      version.start(),
      version.end(),
      version.len(),
      version.range(),
      version.as_str()
    );

    let hash = &pwd[version.end() + 1..];
    (version.as_str().parse().unwrap(), hash)
  } else {
    (0, pwd)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_pwd() -> Result<()> {
    let plain_pwd = "2024.Ultimate";
    let encrypted_pwd = generate_pwd(plain_pwd).await?;
    println!("The pwd is {}", encrypted_pwd);

    assert!(encrypted_pwd.starts_with("#1#"));

    let version = verify_pwd(plain_pwd, &encrypted_pwd).await?;
    assert_eq!(version, 1);

    Ok(())
  }
}
