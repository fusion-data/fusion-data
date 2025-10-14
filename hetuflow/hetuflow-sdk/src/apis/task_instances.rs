//! Task Instances API implementation
use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::page::PageResult;
use hetuflow_core::models::{SchedTaskInstance, TaskInstanceForCreate, TaskInstanceForQuery, TaskInstanceForUpdate};
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Task Instances API client
#[derive(Debug, Clone)]
pub struct TaskInstancesApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for TaskInstancesApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> TaskInstancesApi<'a> {
  /// Create a new task instance
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Query task instances
  pub async fn query(&self, query: TaskInstanceForQuery) -> SdkResult<PageResult<SchedTaskInstance>> {
    let response = self.client.post("task-instances/page", &query).await?;
    Self::handle_response(response).await
  }

  /// Create a new task instance
  pub async fn create(&self, instance: TaskInstanceForCreate) -> SdkResult<Uuid> {
    let response = self.client.post("task-instances/item", &instance).await?;
    Self::handle_response(response).await
  }

  /// Get a task instance by ID
  pub async fn get(&self, id: &Uuid) -> SdkResult<Option<SchedTaskInstance>> {
    let response = self.client.get(&format!("task-instances/item/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// Update a task instance
  pub async fn update(&self, id: &Uuid, update: TaskInstanceForUpdate) -> SdkResult<()> {
    let response = self.client.post(&format!("task-instances/item/{}/update", id), &update).await?;
    Self::handle_response(response).await
  }

  /// Delete a task instance
  pub async fn delete(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.delete(&format!("task-instances/item/{}", id)).await?;
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
