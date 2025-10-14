//! Authentication API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Authentication API client
#[derive(Debug, Clone)]
pub struct AuthApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for AuthApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> AuthApi<'a> {
  /// Creates a new instance of the AuthApi
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Generate a new authentication token
  pub async fn generate_token(&self, request: GenerateTokenRequest) -> SdkResult<GenerateTokenResponse> {
    let response = self.client.post("auth/generate-token", &request).await?;
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

/// Request to generate a new token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTokenRequest {
  /// Agent ID (UUID format)
  pub agent_id: String,
  /// Optional permissions list
  pub permissions: Option<Vec<String>>,
}

/// Response containing the generated token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTokenResponse {
  /// JWE Token
  pub token: String,
  /// Agent ID
  pub agent_id: String,
  /// Token type
  pub token_type: String,
  /// Expiration time (Unix timestamp)
  pub expires_at: i64,
  /// Issued time (ISO 8601 format)
  pub issued_at: String,
}
