//! System API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// System API client
#[derive(Debug, Clone)]
pub struct SystemApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for SystemApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> SystemApi<'a> {
  /// Create a new system API client
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Get system health status
  pub async fn health(&self) -> SdkResult<Value> {
    let response = self.client.get("system/health").await?;
    Self::handle_response(response).await
  }

  /// Get system metrics
  pub async fn metrics(&self) -> SdkResult<Value> {
    let response = self.client.get("system/metrics").await?;
    Self::handle_response(response).await
  }

  async fn handle_response<T: DeserializeOwned>(response: Response) -> SdkResult<T> {
    #[cfg(not(target_arch = "wasm32"))]
    {
      if response.status().is_success() {
        let text = response
          .text()
          .await
          .map_err(|e| SdkError::HttpError(format!("Failed to read response body: {}", e)))?;
        serde_json::from_str(&text).map_err(|e| SdkError::JsonError(format!("JSON decode error: {}", e)))
      } else {
        let status = response.status().as_u16();
        let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err(SdkError::ApiError { status, message })
      }
    }

    #[cfg(target_arch = "wasm32")]
    {
      if response.ok() {
        response.json::<T>().await.map_err(|e| SdkError::from(e))
      } else {
        let status = response.status() as u16;
        let message = format!("HTTP {}: {}", status, response.status_text());
        Err(SdkError::ApiError { status, message })
      }
    }
  }
}
