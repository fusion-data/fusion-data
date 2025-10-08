//! Tasks API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::model::IdUuidResult;
use hetuflow_core::models::{SchedTask, TaskForCreate, TaskForQuery, TaskForUpdate};
use hetuflow_core::types::TaskStatus;
use fusionsql_core::filter::{OpValInt32, OpValUuid};
use fusion_common::page::PageResult;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Tasks API client
#[derive(Debug, Clone)]
pub struct TasksApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for TasksApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> TasksApi<'a> {
  /// Create a new Tasks API client
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  /// Query tasks with pagination and filtering
  pub async fn query(&self, query: TaskForQuery) -> SdkResult<PageResult<SchedTask>> {
    let response = self.client.post("tasks/query", &query).await?;
    Self::handle_response(response).await
  }

  /// Create a new task
  pub async fn create(&self, task: TaskForCreate) -> SdkResult<IdUuidResult> {
    let response = self.client.post("tasks/create", &task).await?;
    Self::handle_response(response).await
  }

  /// Get a task by ID
  pub async fn get(&self, id: &Uuid) -> SdkResult<Option<SchedTask>> {
    let response = self.client.get(&format!("tasks/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// Update an existing task
  pub async fn update(&self, id: &Uuid, update: TaskForUpdate) -> SdkResult<()> {
    let response = self.client.post(&format!("tasks/{}/update", id), &update).await?;
    Self::handle_response(response).await
  }

  /// Retry a failed task
  pub async fn retry(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.post(&format!("tasks/{}/retry", id), &()).await?;
    Self::handle_response(response).await
  }

  /// Cancel a task
  pub async fn cancel(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.post(&format!("tasks/{}/cancel", id), &()).await?;
    Self::handle_response(response).await
  }

  /// Delete a task
  pub async fn delete(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.delete(&format!("tasks/{}", id)).await?;
    Self::handle_response(response).await
  }

  /// List all tasks (simple wrapper around query with default parameters)
  pub async fn list(&self) -> SdkResult<Vec<SchedTask>> {
    let result = self.query(Default::default()).await?;
    Ok(result.result)
  }

  /// Find tasks by job ID
  pub async fn find_by_job(&self, job_id: &Uuid) -> SdkResult<Vec<SchedTask>> {
    let mut query = TaskForQuery::default();
    query.filter.job_id = Some(OpValUuid::eq(*job_id));

    let result = self.query(query).await?;
    Ok(result.result)
  }

  /// Find tasks by status
  pub async fn find_by_status(&self, status: TaskStatus) -> SdkResult<Vec<SchedTask>> {
    let mut query = TaskForQuery::default();
    query.filter.status = Some(OpValInt32::eq(status as i32));

    let result = self.query(query).await?;
    Ok(result.result)
  }

  /// Find tasks by schedule ID
  pub async fn find_by_schedule(&self, schedule_id: &Uuid) -> SdkResult<Vec<SchedTask>> {
    let mut query = TaskForQuery::default();
    query.filter.schedule_id = Some(OpValUuid::eq(*schedule_id));

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
