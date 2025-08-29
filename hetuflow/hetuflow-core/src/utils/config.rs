use std::path::PathBuf;

use config::Config;
use ultimate_core::DataError;

pub fn write_app_config(path: PathBuf, key: &str, id: &str) -> Result<(), DataError> {
  let config = Config::builder()
    .add_source(config::File::from(path.clone()))
    .add_source(config::File::from_str(&format!("{}: {}", key, id), config::FileFormat::Yaml))
    .build()?;

  // Convert config to serde_json::Value
  let config_data: serde_json::Value = config.try_deserialize()?;

  // Serialize to TOML string
  let config_str = toml::to_string_pretty(&config_data)
    .map_err(|e| DataError::server_error(format!("TOML serialization error: {}", e)))?;

  // Write the TOML string to file
  std::fs::write(&path, config_str)?;

  Ok(())
}
