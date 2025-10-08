//! Schedules API implementation

use crate::{
  apis::ApiService,
  error::{SdkError, SdkResult},
  platform::Response,
};
use fusion_common::model::IdUuidResult;
use hetuflow_core::models::{SchedSchedule, ScheduleForCreate, ScheduleForQuery, ScheduleForUpdate};
use fusion_common::page::PageResult;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Schedules API client
#[derive(Debug, Clone)]
pub struct SchedulesApi<'a> {
  client: &'a crate::HetuflowClient,
}

impl<'a> ApiService for SchedulesApi<'a> {
  fn config(&self) -> &crate::Config {
    self.client.config()
  }

  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    self.client.request(method, path, body).await
  }
}

impl<'a> SchedulesApi<'a> {
  pub fn new(client: &'a crate::HetuflowClient) -> Self {
    Self { client }
  }

  pub async fn query(&self, query: ScheduleForQuery) -> SdkResult<PageResult<SchedSchedule>> {
    let response = self.client.post("schedules/page", &query).await?;
    Self::handle_response(response).await
  }

  pub async fn create(&self, schedule: ScheduleForCreate) -> SdkResult<IdUuidResult> {
    let response = self.client.post("schedules/item", &schedule).await?;
    Self::handle_response(response).await
  }

  pub async fn get(&self, id: &Uuid) -> SdkResult<Option<SchedSchedule>> {
    let response = self.client.get(&format!("schedules/item/{}", id)).await?;
    Self::handle_response(response).await
  }

  pub async fn update(&self, id: &Uuid, update: ScheduleForUpdate) -> SdkResult<()> {
    let response = self.client.put(&format!("schedules/item/{}", id), &update).await?;
    Self::handle_response(response).await
  }

  pub async fn delete(&self, id: &Uuid) -> SdkResult<()> {
    let response = self.client.delete(&format!("schedules/item/{}", id)).await?;
    Self::handle_response(response).await
  }

  pub async fn get_schedulable(&self) -> SdkResult<Vec<SchedSchedule>> {
    let response = self.client.get("schedules/schedulable").await?;
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
