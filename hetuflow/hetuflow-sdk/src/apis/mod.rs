//! API modules for Hetuflow services

use crate::{Config, SdkResult, platform::Response};

/// Trait for API service base functionality
pub trait ApiService {
  /// Get the base configuration
  fn config(&self) -> &Config;

  /// Get the base URL
  fn base_url(&self) -> &str {
    &self.config().base_url
  }

  /// Make a GET request
  async fn get(&self, path: &str) -> SdkResult<Response> {
    self.request::<()>("GET", path, None).await
  }

  /// Make a POST request
  async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> SdkResult<Response> {
    self.request("POST", path, Some(body)).await
  }

  /// Make a PUT request
  async fn put<T: serde::Serialize>(&self, path: &str, body: &T) -> SdkResult<Response> {
    self.request("PUT", path, Some(body)).await
  }

  /// Make a DELETE request
  async fn delete(&self, path: &str) -> SdkResult<Response> {
    self.request::<()>("DELETE", path, None).await
  }

  /// Make an HTTP request
  async fn request<T: serde::Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response>;
}

mod agents;
pub use agents::AgentsApi;

mod jobs;
pub use jobs::JobsApi;

mod tasks;
pub use tasks::TasksApi;

mod schedules;
pub use schedules::SchedulesApi;

mod task_instances;
pub use task_instances::TaskInstancesApi;

mod servers;
pub use servers::ServersApi;

mod system;
pub use system::SystemApi;

mod gateway;
pub use gateway::GatewayApi;

mod auth;
pub use auth::AuthApi;
