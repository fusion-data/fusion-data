//! HTTP client for Hetuflow API

use crate::{
  apis::ApiService,
  config::Config,
  error::{SdkError, SdkResult},
  platform::Response,
};
use serde::Serialize;
use std::time::Duration;

/// Main client for interacting with the Hetuflow API
#[derive(Debug, Clone)]
pub struct HetuflowClient {
  config: Config,
  #[cfg(not(target_arch = "wasm32"))]
  http_client: reqwest::Client,
}

impl HetuflowClient {
  /// Create a new client with the given base URL
  pub fn new(base_url: String) -> SdkResult<Self> {
    Self::with_config(Config::new(base_url))
  }

  /// Create a new client with the given configuration
  pub fn with_config(config: Config) -> SdkResult<Self> {
    config.validate()?;

    #[cfg(not(target_arch = "wasm32"))]
    {
      let http_client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| SdkError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

      Ok(Self { config, http_client })
    }

    #[cfg(target_arch = "wasm32")]
    {
      Ok(Self { config })
    }
  }

  /// Get the client configuration
  pub fn config(&self) -> &Config {
    &self.config
  }

  /// Get access to the Agents API
  pub fn agents(&self) -> crate::apis::AgentsApi {
    crate::apis::AgentsApi::new(self)
  }

  /// Get access to the Jobs API
  pub fn jobs(&self) -> crate::apis::JobsApi {
    crate::apis::JobsApi::new(self)
  }

  /// Get access to the Tasks API
  pub fn tasks(&self) -> crate::apis::TasksApi {
    crate::apis::TasksApi::new(self)
  }

  /// Get access to the Schedules API
  pub fn schedules(&self) -> crate::apis::SchedulesApi {
    crate::apis::SchedulesApi::new(self)
  }

  /// Get access to the Task Instances API
  pub fn task_instances(&self) -> crate::apis::TaskInstancesApi {
    crate::apis::TaskInstancesApi::new(self)
  }

  /// Get access to the Servers API
  pub fn servers(&self) -> crate::apis::ServersApi {
    crate::apis::ServersApi::new(self)
  }

  /// Get access to the System API
  pub fn system(&self) -> crate::apis::SystemApi {
    crate::apis::SystemApi::new(self)
  }

  /// Get access to the Gateway API
  pub fn gateway(&self) -> crate::apis::GatewayApi {
    crate::apis::GatewayApi::new(self)
  }

  /// Get access to the Auth API
  pub fn auth(&self) -> crate::apis::AuthApi {
    crate::apis::AuthApi::new(self)
  }
}

impl ApiService for HetuflowClient {
  fn config(&self) -> &Config {
    &self.config
  }

  async fn request<T: Serialize>(&self, method: &str, path: &str, body: Option<&T>) -> SdkResult<Response> {
    let url = format!("{}/api/v1/{}", self.base_url().trim_end_matches('/'), path.trim_start_matches('/'));

    #[cfg(not(target_arch = "wasm32"))]
    {
      let mut request = match method {
        "GET" => self.http_client.get(&url),
        "POST" => self.http_client.post(&url),
        "PUT" => self.http_client.put(&url),
        "DELETE" => self.http_client.delete(&url),
        _ => return Err(SdkError::HttpError(format!("HTTP method {} not supported", method))),
      };

      // Add auth token if available
      if let Some(token) = &self.config.auth_token {
        request = request.bearer_auth(token);
      }

      // Add custom headers
      for (name, value) in &self.config.headers {
        request = request.header(name, value);
      }

      // Add body if provided
      if let Some(body) = body {
        request = request.json(body);
      }

      // Execute request with retry logic
      let mut attempts = 0;
      let max_attempts = self.config.retry_attempts + 1;

      loop {
        let result = request
          .try_clone()
          .unwrap_or_else(|| {
            // Clone failed, reconstruct the request
            let mut req = match method {
              "GET" => self.http_client.get(&url),
              "POST" => self.http_client.post(&url),
              "PUT" => self.http_client.put(&url),
              "DELETE" => self.http_client.delete(&url),
              _ => unreachable!(),
            };
            if let Some(token) = &self.config.auth_token {
              req = req.bearer_auth(token);
            }
            for (name, value) in &self.config.headers {
              req = req.header(name, value);
            }
            if let Some(body) = body {
              req = req.json(body);
            }
            req
          })
          .send()
          .await;

        match result {
          Ok(response) => {
            // Check for rate limiting
            if response.status() == 429 {
              if attempts < max_attempts - 1 {
                let retry_after = response
                  .headers()
                  .get("Retry-After")
                  .and_then(|v| v.to_str().ok())
                  .and_then(|v| v.parse().ok())
                  .unwrap_or(1);

                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                attempts += 1;
                continue;
              } else {
                return Err(SdkError::HttpError(format!(
                  "Rate limited after {} attempts",
                  attempts
                )));
              }
            }

            // Check for server errors that should be retried
            if response.status().is_server_error() && attempts < max_attempts - 1 {
              tokio::time::sleep(self.config.retry_delay).await;
              attempts += 1;
              continue;
            }

            return Ok(response);
          }
          Err(e) if e.is_timeout() || e.is_connect() => {
            if attempts < max_attempts - 1 {
              tokio::time::sleep(self.config.retry_delay * (attempts + 1)).await;
              attempts += 1;
              continue;
            } else {
              return Err(SdkError::NetworkError(e.to_string()));
            }
          }
          Err(e) => return Err(SdkError::from(e)),
        }
      }
    }

    #[cfg(target_arch = "wasm32")]
    {
      use gloo_net::http::{Method, Request, RequestBuilder};
      use web_sys::{AbortController, AbortSignal, RequestCredentials, RequestMode};

      // Create abort controller for timeout handling
      let abort_controller = AbortController::new().ok();
      let abort_signal = abort_controller.as_ref().map(|c| c.signal());

      // Set up timeout if specified
      if let Some(timeout_ms) = self.config.timeout.as_millis().try_into().ok() {
        if let Some(controller) = &abort_controller {
          let controller_clone = controller.clone();
          wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::sleep(Duration::from_millis(timeout_ms)).await;
            controller_clone.abort();
          });
        }
      }

      // 创建请求构建器
      let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => Request::get(&url),
        "POST" => Request::post(&url),
        "PUT" => Request::put(&url),
        "DELETE" => Request::delete(&url),
        "PATCH" => Request::patch(&url),
        _ => {
          // 对于不支持的方法，使用通用的 Request 构建器
          use gloo_net::http::{Method, RequestBuilder};
          let method_enum = match method.to_uppercase().as_str() {
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            _ => Method::GET,
          };
          RequestBuilder::new(&url).method(method_enum)
        }
      };

      // 配置请求模式和凭据
      request_builder = request_builder.mode(RequestMode::Cors).credentials(RequestCredentials::SameOrigin);

      // 添加认证令牌（如果可用）
      if let Some(token) = &self.config.auth_token {
        request_builder = request_builder.header("Authorization", &format!("Bearer {}", token));
      }

      // 添加用户代理
      request_builder = request_builder.header("User-Agent", &self.config.user_agent);

      // 添加压缩头（如果启用）
      if self.config.compression {
        request_builder = request_builder.header("Accept-Encoding", "gzip, deflate");
      }

      // 添加自定义头
      for (name, value) in &self.config.headers {
        request_builder = request_builder.header(name, value);
      }

      // 执行请求并处理重试逻辑
      let mut attempts = 0;
      let max_attempts = self.config.retry_attempts + 1;

      loop {
        // 重新构建请求（因为 send() 会消费 request）
        let request_to_send = if let Some(body) = body {
          let json_body = serde_json::to_string(body).map_err(|e| SdkError::JsonError(e.to_string()))?;

          // 重新构建请求
          let mut new_request_builder = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            "PATCH" => Request::patch(&url),
            "HEAD" => {
              let method_enum = Method::HEAD;
              RequestBuilder::new(&url).method(method_enum)
            }
            "OPTIONS" => {
              let method_enum = Method::OPTIONS;
              RequestBuilder::new(&url).method(method_enum)
            }
            _ => Request::get(&url),
          };

          // 配置请求模式和凭据
          new_request_builder = new_request_builder.mode(RequestMode::Cors).credentials(RequestCredentials::SameOrigin);

          // 添加认证令牌（如果可用）
          if let Some(token) = &self.config.auth_token {
            new_request_builder = new_request_builder.header("Authorization", &format!("Bearer {}", token));
          }

          // 添加用户代理
          new_request_builder = new_request_builder.header("User-Agent", &self.config.user_agent);

          // 添加压缩头（如果启用）
          if self.config.compression {
            new_request_builder = new_request_builder.header("Accept-Encoding", "gzip, deflate");
          }

          // 添加自定义头
          for (name, value) in &self.config.headers {
            new_request_builder = new_request_builder.header(name, value);
          }

          // 设置超时信号（如果可用）
          if abort_signal.is_some() {
            new_request_builder = new_request_builder.abort_signal(abort_signal.as_ref());
          }

          new_request_builder.json(&json_body)?
        } else {
          // 重新构建请求
          let mut new_request_builder = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            "PATCH" => Request::patch(&url),
            "HEAD" => {
              let method_enum = Method::HEAD;
              RequestBuilder::new(&url).method(method_enum)
            }
            "OPTIONS" => {
              let method_enum = Method::OPTIONS;
              RequestBuilder::new(&url).method(method_enum)
            }
            _ => Request::get(&url),
          };

          // 配置请求模式和凭据
          new_request_builder = new_request_builder.mode(RequestMode::Cors).credentials(RequestCredentials::SameOrigin);

          // 添加认证令牌（如果可用）
          if let Some(token) = &self.config.auth_token {
            new_request_builder = new_request_builder.header("Authorization", &format!("Bearer {}", token));
          }

          // 添加用户代理
          new_request_builder = new_request_builder.header("User-Agent", &self.config.user_agent);

          // 添加压缩头（如果启用）
          if self.config.compression {
            new_request_builder = new_request_builder.header("Accept-Encoding", "gzip, deflate");
          }

          // 添加自定义头
          for (name, value) in &self.config.headers {
            new_request_builder = new_request_builder.header(name, value);
          }

          // 设置超时信号（如果可用）
          if abort_signal.is_some() {
            new_request_builder = new_request_builder.abort_signal(abort_signal.as_ref());
          }

          new_request_builder.build()?
        };

        // 发送请求
        match request_to_send.send().await {
          Ok(response) => {
            // 检查是否需要重试
            if response.status() == 429 {
              if attempts < max_attempts {
                let retry_after =
                  response.headers().get("retry-after").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1);

                gloo_timers::future::sleep(Duration::from_secs(retry_after)).await;
                attempts += 1;
                continue;
              }
            }

            // 检查服务器错误（5xx）
            if response.status() >= 500 && response.status() < 600 {
              if attempts < max_attempts {
                let delay = Duration::from_millis(1000 * (2_u64.pow(attempts as u32)));
                gloo_timers::future::sleep(delay).await;
                attempts += 1;
                continue;
              }
            }

            // 返回响应
            return Ok(response);
          }
          Err(e) => {
            // 检查是否为可重试的错误
            let is_retryable = e.to_string().contains("timeout")
              || e.to_string().contains("network")
              || e.to_string().contains("connection");

            if is_retryable && attempts < max_attempts {
              let delay = Duration::from_millis(1000 * (2_u64.pow(attempts as u32)));
              gloo_timers::future::sleep(delay).await;
              attempts += 1;
              continue;
            }

            return Err(SdkError::NetworkError(e.to_string()));
          }
        }
      }
    }
  }
}
