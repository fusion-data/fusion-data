//! WebAssembly bindings for Hetuflow SDK
//!
//! This module provides JavaScript/TypeScript bindings for the Hetuflow client
//! when compiled to WebAssembly using wasm-bindgen.

#![cfg(target_arch = "wasm32")]

use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::apis::AgentsApi;
use crate::{Config, HetuflowClient};

/// Serialization utilities for WASM interop
mod serialization {
  use super::*;
  use wasm_bindgen::JsValue;

  /// Convert a Rust type to JsValue using serde-wasm-bindgen
  pub fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
  }

  /// Convert a JsValue to a Rust type using serde-wasm-bindgen
  pub fn from_js_value<T: for<'de> Deserialize<'de>>(value: &JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value.clone()).map_err(|e| JsError::new(&format!("Deserialization error: {}", e)))
  }

  /// Convert a JsValue to JSON string
  pub fn to_json_string(value: &JsValue) -> Result<String, JsError> {
    match js_sys::JSON::stringify(value) {
      Ok(stringified) => match stringified.as_string() {
        Some(s) => Ok(s),
        None => Err(JsError::new("Failed to convert JSON to string")),
      },
      Err(_) => Err(JsError::new("Failed to stringify value")),
    }
  }

  /// Convert JSON string to JsValue
  pub fn from_json_string(json: &str) -> Result<JsValue, JsError> {
    match js_sys::JSON::parse(json) {
      Ok(value) => Ok(value),
      Err(e) => {
        let error_str = if let Some(s) = e.as_string() { s } else { "JSON parse error".to_string() };
        Err(JsError::new(&format!("JSON parse error: {}", error_str)))
      }
    }
  }
}

/// Initialize console error panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn main() {
  console_error_panic_hook::set_once();
}

/// WebAssembly wrapper for HetuflowClient
///
/// This provides a JavaScript-friendly interface to the Hetuflow client.
/// All async operations return JavaScript Promises.
#[wasm_bindgen]
pub struct WasmHetuflowClient {
  inner: HetuflowClient,
}

#[wasm_bindgen]
impl WasmHetuflowClient {
  /// Create a new Hetuflow client with the given base URL
  ///
  /// # Arguments
  /// * `base_url` - The base URL of the Hetuflow server
  ///
  /// # Returns
  /// A WasmHetuflowClient instance
  ///
  /// # Throws
  /// Throws a JavaScript Error if the client cannot be created
  ///
  /// # Example
  /// ```javascript
  /// import { WasmHetuflowClient } from './hetuflow_sdk.js';
  ///
  /// try {
  ///   const client = new WasmHetuflowClient("http://localhost:8080");
  ///   console.log("Client created successfully");
  /// } catch (error) {
  ///   console.error("Failed to create client:", error);
  /// }
  /// ```
  #[wasm_bindgen(constructor)]
  pub fn new(base_url: String) -> Result<WasmHetuflowClient, JsError> {
    match HetuflowClient::new(base_url) {
      Ok(client) => Ok(WasmHetuflowClient { inner: client }),
      Err(e) => Err(JsError::new(&format!("Failed to create client: {:?}", e))),
    }
  }

  /// Create a new Hetuflow client with the given configuration
  ///
  /// # Arguments
  /// * `config` - A WasmConfig instance
  ///
  /// # Returns
  /// A WasmHetuflowClient instance
  ///
  /// # Throws
  /// Throws a JavaScript Error if the client cannot be created
  #[wasm_bindgen]
  pub fn with_config(config: WasmConfig) -> Result<WasmHetuflowClient, JsError> {
    match HetuflowClient::with_config(config.into()) {
      Ok(client) => Ok(WasmHetuflowClient { inner: client }),
      Err(e) => Err(JsError::new(&format!("Failed to create client: {:?}", e))),
    }
  }

  /// Get access to the Agents API
  ///
  /// # Returns
  /// A WasmAgentsApi instance for managing agents
  #[wasm_bindgen(getter)]
  pub fn agents(&self) -> WasmAgentsApi {
    WasmAgentsApi::new(&self.inner)
  }

  /// Get access to the Jobs API
  ///
  /// # Returns
  /// A WasmJobsApi instance for managing jobs
  #[wasm_bindgen(getter)]
  pub fn jobs(&self) -> WasmJobsApi {
    WasmJobsApi::new(&self.inner)
  }

  /// Get access to the Tasks API
  ///
  /// # Returns
  /// A WasmTasksApi instance for managing tasks
  #[wasm_bindgen(getter)]
  pub fn tasks(&self) -> WasmTasksApi {
    WasmTasksApi::new(&self.inner)
  }

  /// Get access to the Schedules API
  ///
  /// # Returns
  /// A WasmSchedulesApi instance for managing schedules
  #[wasm_bindgen(getter)]
  pub fn schedules(&self) -> WasmSchedulesApi {
    WasmSchedulesApi::new(&self.inner)
  }

  /// Get access to the Task Instances API
  ///
  /// # Returns
  /// A WasmTaskInstancesApi instance for managing task instances
  #[wasm_bindgen(getter)]
  pub fn task_instances(&self) -> WasmTaskInstancesApi {
    WasmTaskInstancesApi::new(&self.inner)
  }

  /// Get access to the Servers API
  ///
  /// # Returns
  /// A WasmServersApi instance for managing servers
  #[wasm_bindgen(getter)]
  pub fn servers(&self) -> WasmServersApi {
    WasmServersApi::new(&self.inner)
  }

  /// Get access to the System API
  ///
  /// # Returns
  /// A WasmSystemApi instance for system operations
  #[wasm_bindgen(getter)]
  pub fn system(&self) -> WasmSystemApi {
    WasmSystemApi::new(&self.inner)
  }

  /// Get access to the Gateway API
  ///
  /// # Returns
  /// A WasmGatewayApi instance for gateway operations
  #[wasm_bindgen(getter)]
  pub fn gateway(&self) -> WasmGatewayApi {
    WasmGatewayApi::new(&self.inner)
  }

  /// Get access to the Auth API
  ///
  /// # Returns
  /// A WasmAuthApi instance for authentication operations
  #[wasm_bindgen(getter)]
  pub fn auth(&self) -> WasmAuthApi {
    WasmAuthApi::new(&self.inner)
  }
}

/// WebAssembly wrapper for Config
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmConfig {
  inner: Config,
}

#[wasm_bindgen]
impl WasmConfig {
  /// Create a new configuration with the given base URL
  ///
  /// # Arguments
  /// * `base_url` - The base URL of the Hetuflow server
  #[wasm_bindgen(constructor)]
  pub fn new(base_url: String) -> WasmConfig {
    WasmConfig { inner: Config::new(base_url) }
  }

  /// Set the authentication token
  ///
  /// # Arguments
  /// * `token` - The authentication token (JWT or other format)
  #[wasm_bindgen(setter)]
  pub fn set_auth_token(&mut self, token: String) {
    self.inner.auth_token = Some(token);
  }

  /// Get the authentication token
  ///
  /// # Returns
  /// The current authentication token, or null if not set
  #[wasm_bindgen(getter)]
  pub fn auth_token(&self) -> Option<String> {
    self.inner.auth_token.clone()
  }

  /// Set a custom header
  ///
  /// # Arguments
  /// * `name` - Header name
  /// * `value` - Header value
  #[wasm_bindgen]
  pub fn set_header(&mut self, name: String, value: String) {
    self.inner.headers.push((name, value));
  }

  /// Set timeout in milliseconds
  ///
  /// # Arguments
  /// * `timeout_ms` - Timeout in milliseconds
  #[wasm_bindgen(setter)]
  pub fn set_timeout(&mut self, timeout_ms: u32) {
    self.inner.timeout = std::time::Duration::from_millis(timeout_ms as u64);
  }

  /// Get timeout in milliseconds
  ///
  /// # Returns
  /// Current timeout in milliseconds
  #[wasm_bindgen(getter)]
  pub fn timeout(&self) -> u32 {
    self.inner.timeout.as_millis() as u32
  }

  /// Enable or disable compression
  ///
  /// # Arguments
  /// * `enabled` - Whether to enable compression
  #[wasm_bindgen(setter)]
  pub fn set_compression(&mut self, enabled: bool) {
    self.inner.compression = enabled;
  }

  /// Get compression setting
  ///
  /// # Returns
  /// Whether compression is enabled
  #[wasm_bindgen(getter)]
  pub fn compression(&self) -> bool {
    self.inner.compression
  }
}

impl From<WasmConfig> for Config {
  fn from(wasm_config: WasmConfig) -> Self {
    wasm_config.inner
  }
}

// API wrapper macro with actual implementation support
macro_rules! declare_api_wrapper {
  ($name:ident, $inner_name:ident, $doc:expr) => {
    #[doc = $doc]
    #[wasm_bindgen]
    pub struct $name {
      // Store the client directly without lifetime parameters
      client: HetuflowClient,
    }

    impl $name {
      fn new(client: &HetuflowClient) -> Self {
        Self { client: client.clone() }
      }
    }

    #[wasm_bindgen]
    impl $name {
      /// Get the API documentation
      /// Returns a string describing the API
      #[wasm_bindgen]
      pub fn docs(&self) -> String {
        $doc.to_string()
      }

      /// Query items with optional parameters
      ///
      /// # Arguments
      /// * `params` - Query parameters (object)
      ///
      /// # Returns
      /// A Promise that resolves to the query result
      ///
      /// # Example
      /// ```javascript
      /// const result = await api.query({ page: 1, limit: 10 });
      /// console.log(result.items);
      /// ```
      #[wasm_bindgen]
      pub fn query(&self, _params: JsValue) -> Promise {
        future_to_promise(async move {
          Err(JsValue::from_str(&format!("{} query method - implementation pending", stringify!($name))))
        })
      }

      /// Get item by ID
      ///
      /// # Arguments
      /// * `id` - Item ID
      ///
      /// # Returns
      /// A Promise that resolves to the item
      #[wasm_bindgen]
      pub fn get(&self, _id: String) -> Promise {
        future_to_promise(async move {
          Err(JsValue::from_str(&format!("{} get method - implementation pending", stringify!($name))))
        })
      }

      /// Create a new item
      ///
      /// # Arguments
      /// * `data` - Item data (object)
      ///
      /// # Returns
      /// A Promise that resolves to the created item
      #[wasm_bindgen]
      pub fn create(&self, _data: JsValue) -> Promise {
        future_to_promise(async move {
          Err(JsValue::from_str(&format!("{} create method - implementation pending", stringify!($name))))
        })
      }

      /// Update an existing item
      ///
      /// # Arguments
      /// * `id` - Item ID
      /// * `data` - Updated item data (object)
      ///
      /// # Returns
      /// A Promise that resolves to the updated item
      #[wasm_bindgen]
      pub fn update(&self, _id: String, _data: JsValue) -> Promise {
        future_to_promise(async move {
          Err(JsValue::from_str(&format!("{} update method - implementation pending", stringify!($name))))
        })
      }

      /// Delete an item
      ///
      /// # Arguments
      /// * `id` - Item ID
      ///
      /// # Returns
      /// A Promise that resolves when the item is deleted
      #[wasm_bindgen]
      pub fn delete(&self, _id: String) -> Promise {
        future_to_promise(async move {
          Err(JsValue::from_str(&format!("{} delete method - implementation pending", stringify!($name))))
        })
      }
    }
  };
}

// Declare WasmAgentsApi with custom implementation
#[wasm_bindgen]
/// API for managing agents in the Hetuflow system
pub struct WasmAgentsApi {
  // Store the client directly without lifetime parameters
  client: HetuflowClient,
}

impl WasmAgentsApi {
  fn new(client: &HetuflowClient) -> Self {
    Self { client: client.clone() }
  }
}

#[wasm_bindgen]
impl WasmAgentsApi {
  /// Query agents with optional parameters
  ///
  /// # Arguments
  /// * `params` - Query parameters (object)
  ///
  /// # Returns
  /// A Promise that resolves to the query result
  ///
  /// # Example
  /// ```javascript
  /// const result = await api.query({ page: 1, limit: 10 });
  /// console.log(result.items);
  /// ```
  #[wasm_bindgen]
  pub fn query(&self, params: JsValue) -> Promise {
    let client = self.client.clone();
    future_to_promise(async move {
      let query = serialization::from_js_value(&params)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse query parameters: {:?}", e)))?;
      let result = client.agents().query(query).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
      serialization::to_js_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {:?}", e)))
    })
  }

  /// Get an agent by ID
  ///
  /// # Arguments
  /// * `id` - Agent ID
  ///
  /// # Returns
  /// A Promise that resolves to the agent
  #[wasm_bindgen]
  pub fn get(&self, id: String) -> Promise {
    let client = self.client.clone();
    future_to_promise(async move {
      let result = client.agents().get(&id).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
      serialization::to_js_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {:?}", e)))
    })
  }

  /// Create a new agent
  ///
  /// # Arguments
  /// * `data` - Agent data (object)
  ///
  /// # Returns
  /// A Promise that resolves to the created agent
  #[wasm_bindgen]
  pub fn create(&self, data: JsValue) -> Promise {
    let client = self.client.clone();
    future_to_promise(async move {
      let agent = serialization::from_js_value(&data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse agent data: {:?}", e)))?;
      let result = client.agents().create(agent).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
      serialization::to_js_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {:?}", e)))
    })
  }

  /// Update an existing agent
  ///
  /// # Arguments
  /// * `id` - Agent ID
  /// * `data` - Updated agent data (object)
  ///
  /// # Returns
  /// A Promise that resolves to the updated agent
  #[wasm_bindgen]
  pub fn update(&self, id: String, data: JsValue) -> Promise {
    let client = self.client.clone();
    future_to_promise(async move {
      let update = serialization::from_js_value(&data)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse update data: {:?}", e)))?;
      let result = client.agents().update(&id, update).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
      serialization::to_js_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {:?}", e)))
    })
  }

  /// Delete an agent
  ///
  /// # Arguments
  /// * `id` - Agent ID
  ///
  /// # Returns
  /// A Promise that resolves when the agent is deleted
  #[wasm_bindgen]
  pub fn delete(&self, id: String) -> Promise {
    let client = self.client.clone();
    future_to_promise(async move {
      client.agents().delete(&id).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
      Ok(JsValue::from_str(&format!("Agent {} deleted successfully", id)))
    })
  }
}

declare_api_wrapper!(WasmJobsApi, JobsApi, "API for managing jobs in the Hetuflow system");
declare_api_wrapper!(WasmTasksApi, TasksApi, "API for managing tasks in the Hetuflow system");
declare_api_wrapper!(WasmSchedulesApi, SchedulesApi, "API for managing schedules in the Hetuflow system");
declare_api_wrapper!(WasmTaskInstancesApi, TaskInstancesApi, "API for managing task instances in the Hetuflow system");
declare_api_wrapper!(WasmServersApi, ServersApi, "API for managing servers in the Hetuflow system");
declare_api_wrapper!(WasmSystemApi, SystemApi, "API for system operations in the Hetuflow system");
declare_api_wrapper!(WasmGatewayApi, GatewayApi, "API for gateway operations in the Hetuflow system");
declare_api_wrapper!(WasmAuthApi, AuthApi, "API for authentication operations in the Hetuflow system");

/// Utility functions for JavaScript interop
#[wasm_bindgen]
pub struct WasmUtils;

#[wasm_bindgen]
impl WasmUtils {
  /// Convert a JavaScript Error to a string
  ///
  /// # Arguments
  /// * `error` - JavaScript Error object
  ///
  /// # Returns
  /// Error message as string
  #[wasm_bindgen]
  pub fn error_to_string(error: &js_sys::Error) -> String {
    error.to_string().into()
  }

  /// Check if a value is a Promise
  ///
  /// # Arguments
  /// * `value` - Any JavaScript value
  ///
  /// # Returns
  /// True if the value is a Promise
  #[wasm_bindgen]
  pub fn is_promise(value: &JsValue) -> bool {
    value.is_instance_of::<js_sys::Promise>()
  }

  /// Get current timestamp in milliseconds
  ///
  /// # Returns
  /// Current timestamp as float
  #[wasm_bindgen]
  pub fn timestamp_ms() -> f64 {
    // Use Date.now() as it's more universally available
    js_sys::Date::now()
  }

  /// Create a safe JSON string from any JavaScript value
  ///
  /// # Arguments
  /// * `value` - Any JavaScript value
  ///
  /// # Returns
  /// JSON string, or error if value cannot be serialized
  #[wasm_bindgen]
  pub fn safe_json_stringify(value: &JsValue) -> Result<String, JsError> {
    match js_sys::JSON::stringify(value) {
      Ok(stringified) => match stringified.as_string() {
        Some(s) => Ok(s),
        None => Err(JsError::new("Failed to convert JSON to string")),
      },
      Err(_) => Err(JsError::new("Failed to stringify value")),
    }
  }
}

/// Error handling utilities
#[wasm_bindgen]
pub struct WasmError;

#[wasm_bindgen]
impl WasmError {
  /// Convert SDK error to JavaScript Error
  ///
  /// # Arguments
  /// * `error` - Error message string
  ///
  /// # Returns
  /// JavaScript Error object
  #[wasm_bindgen]
  pub fn from_sdk_error(error: String) -> JsError {
    JsError::new(&error)
  }

  /// Create a network error
  ///
  /// # Arguments
  /// * `message` - Error message
  ///
  /// # Returns
  /// JavaScript Error object
  #[wasm_bindgen]
  pub fn network_error(message: String) -> JsError {
    JsError::new(&format!("Network Error: {}", message))
  }

  /// Create a validation error
  ///
  /// # Arguments
  /// * `message` - Error message
  ///
  /// # Returns
  /// JavaScript Error object
  #[wasm_bindgen]
  pub fn validation_error(message: String) -> JsError {
    JsError::new(&format!("Validation Error: {}", message))
  }

  /// Create a configuration error
  ///
  /// # Arguments
  /// * `message` - Error message
  ///
  /// # Returns
  /// JavaScript Error object
  #[wasm_bindgen]
  pub fn config_error(message: String) -> JsError {
    JsError::new(&format!("Configuration Error: {}", message))
  }
}
