use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Config missing env: {0}")]
  ConfigMissingEnv(&'static str),

  #[error("Config wrong format, need: {0}")]
  ConfigWrongFormat(&'static str),

  #[error(transparent)]
  ConfigError(#[from] config::ConfigError),

  #[error(transparent)]
  UltimateUtilError(#[from] ultimate_common::Error),
}
