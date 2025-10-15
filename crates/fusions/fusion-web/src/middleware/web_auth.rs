use axum::body::Body;
use fusion_core::application::Application;
use http::{Request, Response, StatusCode, header::CONTENT_TYPE};
use tower_http::auth::AsyncAuthorizeRequest;

use crate::{WebError, extract_ctx};

/// WebAuth is a middleware that checks if the request is authorized.
#[derive(Clone, Default)]
pub struct WebAuth {
  includes: Vec<String>,
  excludes: Vec<String>,
}

impl WebAuth {
  /// Create a new WebAuth middleware.
  ///
  /// # Arguments
  ///
  /// - `includes`: A list of url paths that are must be accessed with authentication.
  /// - `excludes`: A list of url paths that are not must be accessed with authentication.
  pub fn new(includes: Vec<String>, excludes: Vec<String>) -> Self {
    Self { includes, excludes }
  }
}

impl AsyncAuthorizeRequest<Body> for WebAuth {
  type RequestBody = Body;
  type ResponseBody = Body;
  type Future = futures::future::BoxFuture<'static, Result<http::Request<Body>, http::Response<Self::ResponseBody>>>;

  fn authorize(&mut self, request: http::Request<Self::RequestBody>) -> Self::Future {
    let path = request.uri().path();
    if !self.includes.is_empty() && !self.includes.iter().any(|include| path.starts_with(include)) {
      let path = path.to_string();
      return Box::pin(async move {
        Err(web_error_2_body(WebError::new_with_code(401, format!("Url path `{}` is not in includes", path))))
      });
    }

    if !self.excludes.is_empty() && self.excludes.iter().any(|exclude| path.starts_with(exclude)) {
      let path = path.to_string();
      return Box::pin(async move {
        Err(web_error_2_body(WebError::new_with_code(401, format!("Url path `{}` is in excludes", path))))
      });
    }

    Box::pin(async move {
      let (mut parts, body) = request.into_parts();
      let app = Application::global();
      let fusion_config = app.fusion_config();
      let sc = fusion_config.security();
      let ctx = extract_ctx(&parts, sc).map_err(web_error_2_body)?;
      parts.extensions.insert(ctx);
      Ok(Request::from_parts(parts, body))
    })
  }
}

pub fn web_error_2_body(e: WebError) -> Response<Body> {
  let body = serde_json::to_vec(&e).unwrap();
  Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .header(CONTENT_TYPE, "application/json; charset=utf-8")
    .body(Body::from(body))
    .unwrap()
}
