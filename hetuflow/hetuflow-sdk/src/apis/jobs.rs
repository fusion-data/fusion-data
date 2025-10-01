//! Jobs API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::model::IdUuidResult;
use hetuflow_core::models::{JobForCreate, JobForQuery, JobForUpdate, SchedJob};
use fusionsql_core::filter::{OpValString, OpValUuid};
use fusionsql_core::page::PageResult;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Jobs API client
#[derive(Debug, Clone)]
pub struct JobsApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for JobsApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> JobsApi<'a> {
  /// Create a new Jobs API client
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Query jobs with pagination and filtering
  pub async fn query(&self, query: JobForQuery) -> SdkResult<PageResult<SchedJob>> {
    let response = self.client.post("jobs/page", &query).await?;
    Self::handle_response(response).await
  }

  /// Create a new job
  pub async fn create(&self, job: JobForCreate) -> SdkResult<IdUuidResult> {
    let response = self.client.post("jobs/item", &job).await?;
    Self::handle_response(response).await
  }

  /// Get a job by ID
  pub async fn get(&self, id: &Uuid) -> SdkResult<Option<SchedJob>> {
    let response = self.client.get(&format!("jobs/item/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// Update an existing job
  pub async fn update(&self, id: &Uuid, update: JobForUpdate) -> SdkResult<()> {
    let response = self.client.put(&format!("jobs/item/{}", id), &update).await?;
    Self::handle_response(response).await
  }

  /// Enable a job
  pub async fn enable(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.post(&format!("jobs/item/{}/enable", id), &()).await?;
    Self::handle_response(response).await
  }

  /// Disable a job
  pub async fn disable(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.post(&format!("jobs/item/{}/disable", id), &()).await?;
    Self::handle_response(response).await
  }

  /// Delete a job
  pub async fn delete(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.delete(&format!("jobs/item/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// List all jobs (simple wrapper around query with default parameters)
  pub async fn list(&self) -> SdkResult<Vec<SchedJob>> {
    let result = self.query(Default::default()).await?;
    Ok(result.result)
  }

  /// Find jobs by name
  pub async fn find_by_name(&self, name: &str) -> SdkResult<Vec<SchedJob>> {
    let mut query = JobForQuery::default();
    query.filter.name = Some(OpValString::eq(name.to_string()));

    let result = self.query(query).await?;
    Ok(result.result)
  }

  /// Find jobs by namespace
  pub async fn namespace(&self, namespace_id: &str) -> SdkResult<Vec<SchedJob>> {
    let mut query = JobForQuery::default();
    query.filter.namespace_id = Some(OpValString::eq(namespace_id.to_string()));

    let result = self.query(query).await?;
    Ok(result.result)
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
