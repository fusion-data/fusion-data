use serde::{Deserialize, Serialize};
use ultimate_common::{
  string::{deser_str_to_vecu8, ser_vecu8_to_str},
  time::{self, Duration, UtcDateTime},
};

#[derive(Clone, Deserialize, Serialize)]
pub struct SecurityConf {
  pwd: PwdConf,
  token: TokenConf,
}

impl SecurityConf {
  pub fn pwd(&self) -> &PwdConf {
    &self.pwd
  }

  pub fn token(&self) -> &TokenConf {
    &self.token
  }
}

pub trait KeyConf {
  fn secret_key(&self) -> &[u8];
  fn expires_at(&self) -> UtcDateTime;
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PwdConf {
  #[serde(deserialize_with = "deser_str_to_vecu8", serialize_with = "ser_vecu8_to_str")]
  secret_key: Vec<u8>,

  /// 密码过期秒数
  expires_in: i64,

  #[serde(deserialize_with = "deser_str_to_vecu8", serialize_with = "ser_vecu8_to_str")]
  pwd_key: Vec<u8>,

  /// 创建新用户时的默认密码（未未指定）
  default_pwd: String,
}

impl PwdConf {
  pub fn expires_in(&self) -> i64 {
    self.expires_in
  }

  pub fn pwd_key(&self) -> &[u8] {
    &self.pwd_key
  }

  pub fn default_pwd(&self) -> &str {
    &self.default_pwd
  }
}

impl KeyConf for PwdConf {
  fn secret_key(&self) -> &[u8] {
    &self.secret_key
  }

  fn expires_at(&self) -> UtcDateTime {
    time::now_utc() + Duration::seconds(self.expires_in())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenConf {
  #[serde(deserialize_with = "deser_str_to_vecu8", serialize_with = "ser_vecu8_to_str")]
  pub(crate) secret_key: Vec<u8>,

  pub(crate) expires_in: i64,

  #[serde(deserialize_with = "deser_str_to_vecu8", serialize_with = "ser_vecu8_to_str")]
  pub(crate) public_key: Vec<u8>,

  #[serde(deserialize_with = "deser_str_to_vecu8", serialize_with = "ser_vecu8_to_str")]
  pub(crate) private_key: Vec<u8>,
}

impl TokenConf {
  pub fn expires_in(&self) -> i64 {
    self.expires_in
  }

  pub fn public_key(&self) -> &[u8] {
    &self.public_key
  }

  pub fn private_key(&self) -> &[u8] {
    &self.private_key
  }
}

impl KeyConf for TokenConf {
  fn secret_key(&self) -> &[u8] {
    &self.secret_key
  }

  fn expires_at(&self) -> UtcDateTime {
    time::now_utc() + Duration::seconds(self.expires_in())
  }
}
