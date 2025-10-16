use axum::body::Body;
use fusion_common::ctx::Ctx;
use fusion_web::{WebError, middleware::web_error_2_body};
use http::{Request, Response, StatusCode};
use log::{debug, warn};
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

/// Creates a permission-checking middleware layer
pub fn permission_layer<I, S>(permissions: I) -> AsyncRequireAuthorizationLayer<PermissionMiddleware>
where
  I: IntoIterator<Item = S>,
  S: Into<String>,
{
  AsyncRequireAuthorizationLayer::new(PermissionMiddleware::new(permissions))
}

/// Permission checking middleware
#[derive(Clone)]
pub struct PermissionMiddleware {
  permissions: Vec<String>,
}

impl PermissionMiddleware {
  pub fn new<I, S>(permissions: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    Self { permissions: permissions.into_iter().map(|s| s.into()).collect() }
  }
}

impl AsyncAuthorizeRequest<Body> for PermissionMiddleware {
  type RequestBody = Body;
  type ResponseBody = Body;
  type Future = futures::future::BoxFuture<'static, Result<Request<Body>, Response<Self::ResponseBody>>>;

  fn authorize(&mut self, request: Request<Self::RequestBody>) -> Self::Future {
    let permissions = self.permissions.clone();

    Box::pin(async move {
      // Get authentication context
      let ctx = request.extensions().get::<Ctx>().ok_or_else(|| {
        warn!("No authentication context found for permission check");
        web_error_2_body(WebError::new_with_code(
          StatusCode::UNAUTHORIZED.as_u16() as i32,
          "Authentication context required",
        ))
      })?;

      let user_permissions = ctx.payload().get_strings("permissions").unwrap_or_default();

      // Check permissions
      if !permissions.iter().any(|s| user_permissions.contains(&s.as_str())) {
        warn!("Permission denied: user {} missing permissions {:?}", ctx.uid(), permissions);

        return Err(web_error_2_body(WebError::new_with_code(
          StatusCode::FORBIDDEN.as_u16() as i32,
          format!("Permission denied: {:?}", permissions),
        )));
      }

      debug!("Permission check passed: user {} has permission '{:?}'", ctx.uid(), permissions);

      Ok(request)
    })
  }
}
