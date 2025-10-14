//! Servers API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::page::PageResult;
use hetuflow_core::models::{SchedServer, ServerForQuery, ServerForUpdate};
use serde::de::DeserializeOwned;

/// Servers API client
#[derive(Debug, Clone)]
pub struct ServersApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for ServersApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> ServersApi<'a> {
  /// Create a new server
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Query servers
  pub async fn query(&self, query: ServerForQuery) -> SdkResult<PageResult<SchedServer>> {
    let response = self.client.post("servers/query", &query).await?;
    Self::handle_response(response).await
  }

  /// Get a server
  pub async fn get(&self, id: &str) -> SdkResult<Option<SchedServer>> {
    let response = self.client.get(&format!("servers/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// Update a server
  pub async fn update(&self, id: &str, update: ServerForUpdate) -> SdkResult<()> {
    let response = self.client.post(&format!("servers/{}/update", id), &update).await?;
    Self::handle_response(response).await
  }

  /// Delete a server
  pub async fn delete(&self, id: &str) -> SdkResult<()> {
    let response = self.client.delete(&format!("servers/{}", id)).await?;
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
      // use gloo_net::http::Response;
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
