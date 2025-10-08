//! Configuration for the Hetuflow SDK

use std::time::Duration;

/// Configuration options for the Hetuflow client
#[derive(Debug, Clone)]
pub struct Config {
  /// Base URL for the Hetuflow server API
  pub base_url: String,
  /// Authentication token
  pub auth_token: Option<String>,
  /// Request timeout in seconds
  pub timeout: Duration,
  /// Number of retry attempts for failed requests
  pub retry_attempts: u32,
  /// Base delay between retry attempts in milliseconds
  pub retry_delay: Duration,
  /// Maximum delay between retry attempts
  pub retry_max_delay: Duration,
  /// User agent string
  pub user_agent: String,
  /// Additional headers to include in all requests
  pub headers: Vec<(String, String)>,
  /// Whether to enable compression
  pub compression: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      base_url: "http://localhost:8080".to_string(),
      auth_token: None,
      timeout: Duration::from_secs(30),
      retry_attempts: 3,
      retry_delay: Duration::from_millis(1000),
      retry_max_delay: Duration::from_secs(30),
      user_agent: format!("hetuflow-sdk/{}", env!("CARGO_PKG_VERSION")),
      headers: Vec::new(),
      compression: true,
    }
  }
}

impl Config {
  /// Create a new configuration with the given base URL
  pub fn new(base_url: String) -> Self {
    Self { base_url, ..Default::default() }
  }

  /// Set the authentication token
  pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
    self.auth_token = Some(token.into());
    self
  }

  /// Set the request timeout
  pub fn with_timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }

  /// Set retry configuration
  pub fn with_retry_config(mut self, attempts: u32, delay: Duration, max_delay: Duration) -> Self {
    self.retry_attempts = attempts;
    self.retry_delay = delay;
    self.retry_max_delay = max_delay;
    self
  }

  /// Set the user agent string
  pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
    self.user_agent = user_agent.into();
    self
  }

  /// Add a custom header
  pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
    self.headers.push((name.into(), value.into()));
    self
  }

  /// Enable or disable compression
  pub fn with_compression(mut self, enabled: bool) -> Self {
    self.compression = enabled;
    self
  }

  /// Validate the configuration
  pub fn validate(&self) -> Result<(), crate::SdkError> {
    if self.base_url.is_empty() {
      return Err(crate::SdkError::ConfigError("Base URL cannot be empty".to_string()));
    }

    // Validate that base_url is a valid URL
    if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
      return Err(crate::SdkError::ConfigError("Base URL must start with http:// or https://".to_string()));
    }

    Ok(())
  }
}
