use prost::UnknownEnumValue;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Biz error. code: {code}, msg: {msg}")]
  BizError { code: i32, msg: String },
}

impl Error {
  pub fn bad_request(msg: impl Into<String>) -> Self {
    Self::BizError { code: 400, msg: msg.into() }
  }

  pub fn not_found(msg: impl Into<String>) -> Self {
    Self::BizError { code: 404, msg: msg.into() }
  }
}

impl From<UnknownEnumValue> for Error {
  fn from(e: UnknownEnumValue) -> Self {
    Self::BizError { code: 400, msg: e.to_string() }
  }
}

#[cfg(feature = "uuid")]
impl From<uuid::Error> for Error {
  fn from(e: uuid::Error) -> Self {
    Self::BizError { code: 400, msg: e.to_string() }
  }
}
