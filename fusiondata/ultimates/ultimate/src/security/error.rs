use serde::Serialize;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Hmac failure new from slice")]
  HmacFailNewFromSlice,

  #[error("Invalid format")]
  InvalidFormat,

  #[error("Cannot decode ident")]
  CannotDecodeIdent,

  #[error("Cannot decode exp")]
  CannotDecodeExp,

  #[error("Signature not matching")]
  SignatureNotMatching,

  #[error("Exp not iso")]
  ExpNotIso,

  #[error("Expired")]
  Expired,

  #[error(transparent)]
  JoseError(#[from] josekit::JoseError),

  #[error("Failed to hash password")]
  FailedToHashPassword,

  #[error("Invalid password")]
  InvalidPassword,

  #[error("Failed to verify password")]
  FailedToVerifyPassword,
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}
