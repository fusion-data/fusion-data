use std::sync::OnceLock;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash};
use log::{error, trace};
use rand::RngCore;
use regex::Regex;

use super::{Error, INVARIANT_VIOLATED_MSG, RECOMMENDED_LENGTH, Result};

const CUR_PWD_VERSION: u16 = 1;

/// 生成密码。密码格式为：#<version>#<hash>
pub async fn generate_pwd(password: &str) -> Result<String> {
  let hash = try_to_hash(password.to_owned()).await?;
  Ok(format!("#{}#{}", CUR_PWD_VERSION, hash))
}

/// 验证密码。成功返回密码版本，失败返回错误。
pub async fn verify_pwd(password: &str, hashed_pwd: &str) -> Result<u16> {
  let (version, hash) = split_pwd_version(hashed_pwd);
  if verify(password.to_owned(), hash.to_owned()).await? { Ok(version) } else { Err(Error::InvalidPassword) }
}

/// 校验密码复杂度基线：长度≥8，含大小写与数字
pub fn is_strong_password(password: &str) -> bool {
  if password.len() < 8 {
    return false;
  }
  let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
  let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
  let has_digit = password.chars().any(|c| c.is_ascii_digit());
  has_upper && has_lower && has_digit
}

pub(crate) async fn try_to_hash(plain_pwd: String) -> Result<String> {
  tokio::task::spawn_blocking(move || {
    let salt = generate_salt();
    let hash = Argon2::default()
      .hash_password(plain_pwd.as_bytes(), &salt)
      .map_err(|e| {
        error!("Failed to hash password: {}", e);
        Error::FailedToHashPassword
      })?
      .to_string();
    Ok(hash)
  })
  .await
  .unwrap()
}

fn generate_salt() -> SaltString {
  let mut rng = rand::rng();
  let mut bytes = [0u8; RECOMMENDED_LENGTH];
  rng.fill_bytes(&mut bytes);
  SaltString::encode_b64(&bytes).expect(INVARIANT_VIOLATED_MSG)
}

pub(crate) async fn verify(plain_pwd: String, hashed_pwd: String) -> Result<bool> {
  tokio::task::spawn_blocking(move || {
    let hash = PasswordHash::new(&hashed_pwd).map_err(|e| {
      error!("BUG: password hash invalid: {}", e);
      Error::InvalidFormat
    })?;

    let res = Argon2::default().verify_password(plain_pwd.as_bytes(), &hash);

    match res {
      Ok(()) => Ok(true),
      Err(password_hash::Error::Password) => Ok(false),
      Err(e) => {
        error!("Failed to verify password: {}", e);
        Err(Error::FailedToVerifyPassword)
      }
    }
  })
  .await
  .unwrap()
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
    let plain_pwd = "2024.Fusion";
    let encrypted_pwd = generate_pwd(plain_pwd).await?;
    println!("The pwd is {}", encrypted_pwd);

    assert!(encrypted_pwd.starts_with("#1#"));

    let version = verify_pwd(plain_pwd, &encrypted_pwd).await?;
    assert_eq!(version, 1);

    Ok(())
  }

  #[test]
  fn test_password_strength() {
    // 弱密码 - 长度不足
    assert!(!is_strong_password("Ab1"));
    assert!(!is_strong_password("Abcdef1"));

    // 弱密码 - 缺少大写字母
    assert!(!is_strong_password("abcdefg1"));
    assert!(!is_strong_password("abcdefgh1"));

    // 弱密码 - 缺少小写字母
    assert!(!is_strong_password("ABCDEFG1"));
    assert!(!is_strong_password("ABCDEFGH1"));

    // 弱密码 - 缺少数字
    assert!(!is_strong_password("Abcdefgh"));
    assert!(!is_strong_password("AbcdefghIj"));

    // 强密码 - 满足所有条件
    assert!(is_strong_password("Abcdefg1"));
    assert!(is_strong_password("Password123"));
    assert!(is_strong_password("MySecret1"));
    assert!(is_strong_password("TestPass1"));
  }
}
