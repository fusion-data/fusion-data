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
  fn get(&self, path: &str) -> impl Future<Output = SdkResult<Response>> {
    self.request::<()>("GET", path, None)
  }

  /// Make a POST request
  fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> impl Future<Output = SdkResult<Response>> {
    self.request("POST", path, Some(body))
  }

  /// Make a PUT request
  fn put<T: serde::Serialize>(&self, path: &str, body: &T) -> impl Future<Output = SdkResult<Response>> {
    self.request("PUT", path, Some(body))
  }

  /// Make a DELETE request
  fn delete(&self, path: &str) -> impl Future<Output = SdkResult<Response>> {
    self.request::<()>("DELETE", path, None)
  }

  /// Make an HTTP request
  fn request<T: serde::Serialize>(
    &self,
    method: &str,
    path: &str,
    body: Option<&T>,
  ) -> impl Future<Output = SdkResult<Response>>;
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
