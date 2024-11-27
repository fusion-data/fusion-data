use thiserror::Error;

use crate::DataError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Component not found, name is {0}")]
  ComponentNotFound(String),

  #[error("Component type mismatch, type is {0}")]
  ComponentTypeMismatch(&'static str),
}

impl From<Error> for DataError {
  fn from(value: Error) -> Self {
    DataError::InternalError { code: 500, msg: value.to_string(), cause: Some(Box::new(value)) }
  }
}
