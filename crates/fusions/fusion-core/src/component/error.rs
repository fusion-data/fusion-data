use thiserror::Error;

use crate::DataError;

pub type ComponentResult<T> = core::result::Result<T, ComponentError>;

#[derive(Debug, Error)]
pub enum ComponentError {
  #[error("Component not found, name is {0}")]
  ComponentNotFound(String),

  #[error("Component type mismatch, type is {0}")]
  ComponentTypeMismatch(&'static str),
}

impl From<ComponentError> for DataError {
  fn from(value: ComponentError) -> Self {
    DataError::InternalError { code: 500, msg: value.to_string(), cause: Some(Box::new(value)) }
  }
}
