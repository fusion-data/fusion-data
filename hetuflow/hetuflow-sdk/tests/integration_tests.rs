//! Integration tests for the Hetuflow SDK

use fusion_common::page::Page;
use hetuflow_core::models::{
  AgentCapabilities, AgentFilter, AgentForCreate, AgentForQuery, ExecuteCommand, JobForCreate, TaskConfig,
};
use hetuflow_core::types::{AgentStatus, JobStatus};
use hetuflow_sdk::{Config, HetuflowClient};

#[tokio::test]
async fn test_client_creation() {
  let config = Config::new("http://localhost:9500".to_string());
  let client = HetuflowClient::with_config(config);
  assert!(client.is_ok());
}

#[tokio::test]
async fn test_config_validation() {
  // Valid config
  let valid_config = Config::new("http://localhost:9500".to_string());
  assert!(HetuflowClient::with_config(valid_config).is_ok());

  // Invalid config - empty URL
  let invalid_config = Config::new("".to_string());
  assert!(HetuflowClient::with_config(invalid_config).is_err());

  // Invalid config - malformed URL
  let invalid_config = Config::new("not-a-url".to_string());
  assert!(HetuflowClient::with_config(invalid_config).is_err());
}

#[tokio::test]
async fn test_api_clients_creation() {
  let client = HetuflowClient::new("http://localhost:9500".to_string()).unwrap();

  // Test that all API clients can be created
  let _agents = client.agents();
  let _jobs = client.jobs();
  let _tasks = client.tasks();
  let _schedules = client.schedules();
  let _task_instances = client.task_instances();
  let _servers = client.servers();
  let _system = client.system();
  let _gateway = client.gateway();
  let _auth = client.auth();
}

#[test]
fn test_model_creation() {
  // Test that we can create models
  let _agent_create = AgentForCreate {
    id: "test-agent".to_string(),
    description: Some("Test agent".to_string()),
    host: "localhost".to_string(),
    port: 8080,
    status: AgentStatus::Online,
    capabilities: AgentCapabilities {
      max_concurrent_tasks: 10,
      labels: Default::default(),
      metadata: Default::default(),
    },
  };

  let _job_create = JobForCreate {
    id: None,
    namespace_id: Some("default".to_string()),
    name: "test-job".to_string(),
    description: Some("Test job".to_string()),
    environment: Some(serde_json::json!({"ENV": "test"})),
    config: Some(TaskConfig {
      timeout: 300,
      max_retries: 3,
      retry_interval: 60,
      cmd: ExecuteCommand::Bash,
      args: vec!["echo".to_string(), "test".to_string()],
      capture_output: true,
      max_output_size: 1024 * 1024,
      labels: None,
      resource_limits: None,
    }),
    status: Some(JobStatus::Enabled),
  };

  let _query = AgentForQuery {
    filter: AgentFilter::default(),
    page: Page { page: Some(1), limit: Some(10), offset: Some(0), order_bys: None },
  };
}

#[test]
fn test_error_types() {
  use hetuflow_sdk::{SdkError, SdkResult};

  let _: SdkResult<()> = Err(SdkError::NetworkError("test".to_string()));
  let _: SdkResult<()> = Err(SdkError::ApiError { status: 404, message: "Not found".to_string() });
  let _: SdkResult<()> = Err(SdkError::AuthError("Unauthorized".to_string()));
  let _: SdkResult<()> = Err(SdkError::ValidationError("Invalid input".to_string()));
  let _: SdkResult<()> = Err(SdkError::Unknown("Unknown error".to_string()));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_wasm_client_creation() {
  use hetuflow_sdk::HetuflowClient;

  let client = HetuflowClient::new("http://localhost:9500".to_string());
  assert!(client.is_ok());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn test_wasm_api_clients() {
  use hetuflow_sdk::HetuflowClient;

  let client = HetuflowClient::new("http://localhost:8080").unwrap();

  // Test that all API clients can be created
  let _agents = client.agents();
  let _jobs = client.jobs();
  let _tasks = client.tasks();
  let _schedules = client.schedules();
  let _task_instances = client.task_instances();
  let _servers = client.servers();
  let _system = client.system();
  let _gateway = client.gateway();
  let _auth = client.auth();
}
