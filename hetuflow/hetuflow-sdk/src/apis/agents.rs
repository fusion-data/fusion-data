//! Agents API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::model::IdStringResult;
use fusion_common::page::PageResult;
use fusionsql_core::filter::{OpValInt32, OpValString};
use hetuflow_core::models::{AgentFilter, AgentForCreate, AgentForQuery, AgentForUpdate, SchedAgent};
use hetuflow_core::types::AgentStatus;
use serde::de::DeserializeOwned;

/// Agents API client
#[derive(Debug, Clone)]
pub struct AgentsApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for AgentsApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> AgentsApi<'a> {
  /// Create a new Agents API client
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Query agents with pagination and filtering
  pub async fn query(&self, query: AgentForQuery) -> SdkResult<PageResult<SchedAgent>> {
    let response = self.client.post("agents/query", &query).await?;
    Self::handle_response(response).await
  }

  /// Create a new agent
  pub async fn create(&self, agent: AgentForCreate) -> SdkResult<IdStringResult> {
    let response = self.client.post("agents/create", &agent).await?;
    Self::handle_response(response).await
  }

  /// Get an agent by ID
  pub async fn get(&self, id: &str) -> SdkResult<Option<SchedAgent>> {
    let response = self.client.get(&format!("agents/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// Update an existing agent
  pub async fn update(&self, id: &str, update: AgentForUpdate) -> SdkResult<()> {
    let response = self.client.put(&format!("agents/{}", id), &update).await?;
    Self::handle_response(response).await
  }

  /// Delete an agent
  pub async fn delete(&self, id: &str) -> SdkResult<()> {
    let response = self.client.delete(&format!("agents/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// List all agents (simple wrapper around query with default parameters)
  pub async fn list(&self) -> SdkResult<Vec<SchedAgent>> {
    let response = self.client.get("agents").await?;
    Self::handle_response(response).await
  }

  /// Find agents by status
  pub async fn find_by_status(&self, status: AgentStatus) -> SdkResult<PageResult<SchedAgent>> {
    let query = AgentForQuery {
      filter: AgentFilter { status: Some(OpValInt32::eq(status as i32)), ..Default::default() },
      page: Default::default(),
    };
    self.query(query).await
  }

  /// Find agents by address
  pub async fn find_by_address(&self, address: &str) -> SdkResult<PageResult<SchedAgent>> {
    let query = AgentForQuery {
      filter: AgentFilter { address: Some(OpValString::eq(address)), ..Default::default() },
      page: Default::default(),
    };
    self.query(query).await
  }

  /// Handle HTTP response and parse JSON
  async fn handle_response<T: DeserializeOwned>(response: Response) -> SdkResult<T> {
    #[cfg(not(target_arch = "wasm32"))]
    {
      let status = response.status().as_u16();

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
