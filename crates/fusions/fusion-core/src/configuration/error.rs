use thiserror::Error;

pub type ConfigureResult<T> = core::result::Result<T, ConfigureError>;

#[derive(Error, Debug)]
pub enum ConfigureError {
  #[error("Config missing env: {0}")]
  ConfigMissingEnv(&'static str),

  #[error("Config wrong format, need: {0}")]
  ConfigWrongFormat(&'static str),

  #[error(transparent)]
  ConfigError(#[from] config::ConfigError),
}
