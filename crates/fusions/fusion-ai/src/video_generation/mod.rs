//! rig_video_providers
//!
//! Unified VideoGenerationProvider trait with three provider implementations:
//! - openai (feature "openai")
//! - siliconflow (feature "siliconflow")
//! - gitee (feature "gitee")
//!
//! A Router is provided to select which provider to call at runtime.
//!
//! This crate is prepared as a rig-core-style provider library (compatible with rig-core patterns).

use ahash::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for provider operations.
#[derive(Debug, Error)]
pub enum VideoGenerationError {
  #[error("HTTP error: {0}")]
  Http(#[from] reqwest::Error),

  #[error("API error: {0}")]
  Api(String),

  #[error("Serde error: {0}")]
  Serde(#[from] serde_json::Error),

  #[error("Other: {0}")]
  Other(String),
}

/// Request: includes model, prompt, optional duration and a `provider` hint.
/// The `provider` field can be used by Router to pick a provider; if omitted Router uses default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoGenerationRequest {
  pub model: String,
  pub prompt: String,
  pub duration: Option<u32>,
  pub size: Option<String>,
  /// Optional provider hint: "openai", "siliconflow", "gitee"
  pub provider: Option<String>,
  /// Provider-specific options can be put here
  pub options: Option<serde_json::Value>,
}

/// Response from status check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoGenerationResponse {
  pub ready: bool,
  pub video_url: Option<String>,
  pub meta: Option<serde_json::Value>,
}

/// Provider trait: submit returns a request_id (job id). check_status polls for completion.
#[async_trait]
pub trait VideoGenerationProvider: Send + Sync + 'static {
  async fn submit(&self, req: VideoGenerationRequest) -> Result<String, VideoGenerationError>;
  async fn check_status(&self, request_id: &str) -> Result<VideoGenerationResponse, VideoGenerationError>;
}

/// Router that holds named providers and delegates calls.
pub struct ProviderRouter {
  providers: HashMap<String, Box<dyn VideoGenerationProvider>>,
  default: String,
}

impl ProviderRouter {
  pub fn new(default: impl Into<String>) -> Self {
    Self { providers: HashMap::default(), default: default.into() }
  }

  pub fn register<P: VideoGenerationProvider>(&mut self, name: impl Into<String>, provider: P) {
    self.providers.insert(name.into(), Box::new(provider));
  }

  fn select_provider<'a>(
    &'a self,
    hint: Option<&str>,
  ) -> Result<&'a Box<dyn VideoGenerationProvider>, VideoGenerationError> {
    if let Some(h) = hint {
      if let Some(p) = self.providers.get(h) {
        return Ok(p);
      } else {
        return Err(VideoGenerationError::Other(format!("provider hint '{}' not registered", h)));
      }
    }
    // fallback default
    self
      .providers
      .get(&self.default)
      .ok_or(VideoGenerationError::Other(format!("default provider '{}' not registered", self.default)))
  }
}

#[async_trait]
impl VideoGenerationProvider for ProviderRouter {
  async fn submit(&self, req: VideoGenerationRequest) -> Result<String, VideoGenerationError> {
    let hint = req.provider.as_deref();
    let p = self.select_provider(hint)?;
    p.submit(req).await
  }

  async fn check_status(&self, request_id: &str) -> Result<VideoGenerationResponse, VideoGenerationError> {
    // Here we cannot know which provider owns the request_id.
    // For demonstration, we try all providers until one returns a non-error.
    for (_name, prov) in &self.providers {
      match prov.check_status(request_id).await {
        Ok(resp) => return Ok(resp),
        Err(_) => continue,
      }
    }
    Err(VideoGenerationError::Other("no provider recognized the request_id".into()))
  }
}

//
// Provider modules
//
pub mod gitee;
pub mod openai;
pub mod siliconflow;

// ---------------------- Utilities ----------------------
/// Simple helper: poll until ready with exponential backoff up to max_attempts.
pub async fn poll_until_ready<P: VideoGenerationProvider + ?Sized>(
  provider: &P,
  request_id: &str,
  interval_secs: u64,
  max_attempts: usize,
) -> Result<VideoGenerationResponse, VideoGenerationError> {
  let mut attempts = 0usize;
  loop {
    attempts += 1;
    let status = provider.check_status(request_id).await?;
    if status.ready {
      return Ok(status);
    }
    if attempts >= max_attempts {
      return Err(VideoGenerationError::Other(format!("max attempts reached ({})", max_attempts)));
    }
    tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
  }
}
