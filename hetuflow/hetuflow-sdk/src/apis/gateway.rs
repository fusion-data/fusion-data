//! Gateway API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::model::IdUuidResult;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Gateway API client
#[derive(Debug, Clone)]
pub struct GatewayApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for GatewayApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> GatewayApi<'a> {
  /// Creates a new instance of the GatewayApi
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Send a command through the gateway
  pub async fn send_command(&self, command: Value) -> SdkResult<IdUuidResult> {
    let response = self.client.post("gateway/command", &command).await?;
    Self::handle_response(response).await
  }

  /// Get WebSocket connection URL for an agent
  pub fn websocket_url(&self, agent_id: &str) -> String {
    format!("{}/api/v1/gateway/ws?agent_id={}", self.client.base_url().trim_end_matches('/'), agent_id)
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
      use gloo_net::http::Response;
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
