use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use ultimate_common::model::sensitive::UriString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConf {
  enable: bool,

  /// The URI of the database
  // #[serde(serialize_with = "ser_str_secret")]
  url: Option<UriString>,

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

  /// Maximum idle time for a particular connection to prevent
  /// network resource exhaustion
  idle_timeout: Option<Duration>,

  /// Set the maximum amount of time to spend waiting for acquiring a connection
  acquire_timeout: Option<Duration>,

  /// Set the maximum lifetime of individual connections
  max_lifetime: Option<Duration>,

  /// Enable SQLx statement logging
  sqlx_logging: Option<bool>,

  /// SQLx statement logging level (ignored if `sqlx_logging` is false)
  sqlx_logging_level: Option<String>,

  /// set sqlcipher key
  sqlcipher_key: Option<Cow<'static, str>>,

  /// Schema search path (PostgreSQL only)
  schema_search_path: Option<String>,
}

impl DbConf {
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
}
