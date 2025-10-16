use axum::body::Body;
use fusion_common::ctx::Ctx;
use fusion_web::{WebError, middleware::web_error_2_body};
use http::{Request, Response, StatusCode};
use log::{debug, warn};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

/// Creates a tenant isolation middleware layer
pub fn tenant_middleware_layer() -> AsyncRequireAuthorizationLayer<TenantMiddleware> {
  AsyncRequireAuthorizationLayer::new(TenantMiddleware)
}

/// Tenant isolation middleware
#[derive(Clone)]
pub struct TenantMiddleware;

impl TenantMiddleware {
  pub fn new() -> Self {
    Self
  }
}

impl Default for TenantMiddleware {
  fn default() -> Self {
    Self::new()
  }
}

impl AsyncAuthorizeRequest<Body> for TenantMiddleware {
  type RequestBody = Body;
  type ResponseBody = Body;
  type Future = futures::future::BoxFuture<'static, Result<Request<Body>, Response<Self::ResponseBody>>>;

  fn authorize(&mut self, request: Request<Self::RequestBody>) -> Self::Future {
    Box::pin(async move {
      // Get authentication context
      let ctx = request.extensions().get::<Ctx>().ok_or_else(|| {
        warn!("No authentication context found for tenant isolation");
        web_error_2_body(WebError::new_with_code(
          StatusCode::UNAUTHORIZED.as_u16() as i32,
          "Authentication context required",
        ))
      })?;

      // Extract tenant_id from token payload
      let tenant_id = ctx.get_tenant_id().ok_or_else(|| {
        warn!("No tenant_id found in authentication context");
        web_error_2_body(WebError::new_with_code(StatusCode::UNAUTHORIZED.as_u16() as i32, "Tenant context required"))
      })?;

      debug!("Tenant isolation check passed: user {} accessing tenant {}", ctx.uid(), tenant_id);

      // Store tenant_id in request extensions for later use
      let mut request = request;
      request.extensions_mut().insert(TenantContext { tenant_id });

      Ok(request)
    })
  }
}

/// Tenant context for request-scoped tenant information
#[derive(Clone, Debug)]
pub struct TenantContext {
  pub tenant_id: i64,
}

/// Extension trait to extract tenant context from HTTP requests
pub trait RequestTenantExt {
  /// Get the tenant context from the request extensions
  fn tenant(&self) -> Option<&TenantContext>;
}

impl RequestTenantExt for http::Request<Body> {
  fn tenant(&self) -> Option<&TenantContext> {
    self.extensions().get::<TenantContext>()
  }
}

/// Extension trait to extract tenant context from Axum request parts
pub trait RequestPartsTenantExt {
  /// Get the tenant context from the request parts extensions
  fn tenant(&self) -> Option<&TenantContext>;
}

impl RequestPartsTenantExt for axum::http::request::Parts {
  fn tenant(&self) -> Option<&TenantContext> {
    self.extensions.get::<TenantContext>()
  }
}
