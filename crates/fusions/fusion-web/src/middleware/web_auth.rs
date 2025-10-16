use std::sync::Arc;
use std::time::SystemTime;

use axum::body::Body;
use http::request::Parts;
use http::{Request, header::AUTHORIZATION, header::CONTENT_TYPE};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

use fusion_common::ctx::Ctx;
use fusion_core::application::Application;

use crate::extract_ctx;
use crate::{WebError, middleware::web_error_2_body};

#[derive(Clone)]
struct ExternalUrl {
  url: Arc<String>,
  client: reqwest::Client,
}

/// WebAuth is a middleware that checks if the request is authorized by calling remote API.
#[derive(Clone, Default)]
pub struct WebAuth {
  /// A list of url paths that must be accessed with authentication.
  includes: Arc<Vec<String>>,
  /// A list of url paths that must not be accessed with authentication.
  excludes: Arc<Vec<String>>,
  external: Option<ExternalUrl>,
}

impl WebAuth {
  /// Set the API base URL.
  pub fn with_api_base_url(mut self, api_base_url: &str) -> Self {
    self.external = Some(ExternalUrl {
      url: Arc::new(format!("{}/api/auth/extract_token", api_base_url.trim_end_matches('/'))),
      client: reqwest::Client::new(),
    });
    self
  }

  /// Add includes paths.
  pub fn with_includes(mut self, includes: Vec<String>) -> Self {
    self.includes = Arc::new(includes);
    self
  }

  /// Add excludes paths.
  pub fn with_excludes(mut self, excludes: Vec<String>) -> Self {
    self.excludes = Arc::new(excludes);
    self
  }

  pub fn into_layer(self) -> AsyncRequireAuthorizationLayer<Self> {
    AsyncRequireAuthorizationLayer::new(self)
  }
}

impl AsyncAuthorizeRequest<Body> for WebAuth {
  type RequestBody = Body;
  type ResponseBody = Body;
  type Future = futures::future::BoxFuture<'static, Result<http::Request<Body>, http::Response<Self::ResponseBody>>>;

  fn authorize(&mut self, request: http::Request<Self::RequestBody>) -> Self::Future {
    let path = request.uri().path().to_string();
    // Check if path is in includes (if includes is not empty)
    if !self.includes.is_empty() && !self.includes.iter().any(|include| path.starts_with(include)) {
      return Box::pin(async move {
        Err(web_error_2_body(WebError::new_with_code(401, format!("Url path `{}` is not in includes", path))))
      });
    }

    // Check if path is in excludes (if excludes is not empty)
    if !self.excludes.is_empty() && self.excludes.iter().any(|exclude| path.starts_with(exclude)) {
      return Box::pin(async move {
        Err(web_error_2_body(WebError::new_with_code(401, format!("Url path `{}` is in excludes", path))))
      });
    }

    match self.external.as_ref() {
      Some(external) => {
        let url = external.url.clone();
        let client = external.client.clone();
        Box::pin(async move {
          let (mut parts, body) = request.into_parts();
          let ctx = validate_token_remote(&url, &client, &parts).await.map_err(web_error_2_body)?;
          parts.extensions.insert(ctx);
          Ok(Request::from_parts(parts, body))
        })
      }
      None => Box::pin(async {
        let (mut parts, body) = request.into_parts();
        let ctx = extract_ctx(&parts, Application::global().fusion_setting().security()).map_err(web_error_2_body)?;
        parts.extensions.insert(ctx);
        Ok(Request::from_parts(parts, body))
      }),
    }
  }
}

/// Call remote API to validate token and extract context
async fn validate_token_remote(url: &str, client: &reqwest::Client, parts: &Parts) -> Result<Ctx, WebError> {
  let header_value = parts
    .headers
    .get(AUTHORIZATION)
    .ok_or_else(|| WebError::new_with_code(401, "Missing authentication token"))?;

  let response = client
    .post(url)
    .header(AUTHORIZATION, header_value)
    .header(CONTENT_TYPE, "application/json")
    .send()
    .await
    .map_err(|e| WebError::new_with_code(401, format!("Failed to call auth API: {}", e)))?;

  if response.status().is_success() {
    let response_data: serde_json::Value = response
      .json()
      .await
      .map_err(|e| WebError::new_with_code(401, format!("Failed to parse auth response: {}", e)))?;

    // Convert response data to CtxPayload
    let response_map = response_data
      .as_object()
      .ok_or_else(|| WebError::new_with_code(401, "Invalid token response format"))?
      .clone();

    let ctx_payload = fusion_common::ctx::CtxPayload::from(response_map);

    // Create Ctx from payload
    let req_time = SystemTime::now();
    let ctx = Ctx::try_new(ctx_payload, Some(req_time), fusion_core::log::get_trace_id())
      .map_err(|e| WebError::new_with_code(401, format!("Failed to create context: {}", e)))?;

    Ok(ctx)
  } else {
    let status = response.status();
    let error_msg = format!("Token validation failed: {}", status);
    Err(WebError::new_with_code(401, error_msg))
  }
}
