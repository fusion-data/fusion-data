use std::borrow::Cow;

use duration_str::deserialize_option_duration;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::common::UriString;
use crate::store::dbx::DbxProvider;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DbConfig {
  enable: bool,

  /// The URI of the database
  url: Option<UriString>,

  /// The type of the database, available values are `postgres` and `sqlite`.
  /// When not using a `url`, this value needs to be set.
  provider: Option<DbxProvider>,
  host: Option<String>,
  port: Option<u16>,
  socket: Option<String>,
  database: Option<String>,
  username: Option<String>,
  #[serde(skip_serializing)]
  password: Option<String>,

  /// Maximum number of connections for a pool
  max_connections: Option<u32>,

  /// Minimum number of connections for a pool
  min_connections: Option<u32>,

  /// Maximum idle time for a particular connection to prevent network resource exhaustion
  #[serde(default, deserialize_with = "deserialize_option_duration")]
  idle_timeout: Option<Duration>,

  /// Set the maximum amount of time to spend waiting for acquiring a connection
  #[serde(default, deserialize_with = "deserialize_option_duration")]
  acquire_timeout: Option<Duration>,

  /// Set the query to execute after connecting to the database
  after_connect: Option<String>,

  /// Set the maximum lifetime of individual connections
  #[serde(default, deserialize_with = "deserialize_option_duration")]
  max_lifetime: Option<Duration>,

  /// Enable SQLx statement logging
  sqlx_logging: Option<bool>,

  /// SQLx statement logging level (ignored if `sqlx_logging` is false)
  sqlx_logging_level: Option<String>,

  /// set sqlcipher key
  sqlcipher_key: Option<Cow<'static, str>>,

  /// Schema search path (PostgreSQL only)
  schema_search_path: Option<String>,

  application_name: Option<String>,
}

impl DbConfig {
  pub fn enable(&self) -> bool {
    self.enable
  }
  pub fn url(&self) -> Option<&str> {
    self.url.as_deref()
  }

  pub fn host(&self) -> Option<&str> {
    self.host.as_deref()
  }

  pub fn port(&self) -> Option<u16> {
    self.port
  }

  pub fn socket(&self) -> Option<&str> {
    self.socket.as_deref()
  }

  pub fn database(&self) -> Option<&str> {
    self.database.as_deref()
  }

  pub fn username(&self) -> Option<&str> {
    self.username.as_deref()
  }

  pub fn password(&self) -> Option<&str> {
    self.password.as_deref()
  }

  pub fn max_connections(&self) -> Option<u32> {
    self.max_connections
  }

  pub fn min_connections(&self) -> Option<u32> {
    self.min_connections
  }

  pub fn idle_timeout(&self) -> Option<&Duration> {
    self.idle_timeout.as_ref()
  }

  pub fn acquire_timeout(&self) -> Option<&Duration> {
    self.acquire_timeout.as_ref()
  }

  pub fn max_lifetime(&self) -> Option<&Duration> {
    self.max_lifetime.as_ref()
  }

  pub fn after_connect(&self) -> Option<&str> {
    self.after_connect.as_deref()
  }

  pub fn sqlx_logging(&self) -> Option<bool> {
    self.sqlx_logging
  }

  pub fn sqlx_logging_level(&self) -> Option<&str> {
    self.sqlx_logging_level.as_deref()
  }

  pub fn sqlcipher_key(&self) -> Option<&str> {
    self.sqlcipher_key.as_deref()
  }

  pub fn schema_search_path(&self) -> Option<&str> {
    self.schema_search_path.as_deref()
  }

  pub fn application_name(&self) -> Option<&str> {
    self.application_name.as_deref()
  }

  pub fn dbx_type(&self) -> DbxProvider {
    if let Some(provider) = self.provider {
      return provider;
    }

    if let Some(url) = self.url() {
      #[cfg(feature = "with-postgres")]
      if url.starts_with("postgresql://") || url.starts_with("postgres://") {
        return DbxProvider::Postgres;
      }
      #[cfg(feature = "with-sqlite")]
      if url.starts_with("file:") || url.starts_with("sqlite:") {
        return DbxProvider::Sqlite;
      }
    }

    panic!("Unsupported database type: {:?}", self);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_db_config() {
    println!("test_db_config");
    let config = DbConfig {
      enable: true,
      url: Some("postgres://user:pass@localhost:5432/db".into()),
      idle_timeout: Some(Duration::from_secs(10)),
      acquire_timeout: Some(Duration::from_secs(10)),
      max_lifetime: Some(Duration::from_secs(10)),
      sqlx_logging: Some(true),
      sqlx_logging_level: Some("debug".into()),
      ..Default::default()
    };
    let value = serde_json::to_value(config).unwrap();
    let json_str = serde_json::to_string_pretty(&value).unwrap();
    eprintln!("{}", json_str);
    assert!(!json_str.is_empty());
  }
}
