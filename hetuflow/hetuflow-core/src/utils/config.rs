use std::path::PathBuf;

use config::Config;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error(transparent)]
  ConfigError(#[from] config::ConfigError),
  #[error(transparent)]
  TomlError(#[from] toml::ser::Error),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
}

pub fn write_app_config(path: PathBuf, key: &str, id: &str) -> Result<(), ConfigError> {
  let config = Config::builder()
    .add_source(config::File::from(path.clone()))
    .add_source(config::File::from_str(&format!("{}: {}", key, id), config::FileFormat::Yaml))
    .build()?;

  // Convert config to serde_json::Value
  let config_data: serde_json::Value = config.try_deserialize()?;

  // Serialize to TOML string
  let config_str = toml::to_string_pretty(&config_data)?;

  // Write the TOML string to file
  std::fs::write(&path, config_str)?;

  Ok(())
}
