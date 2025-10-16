use axum::{extract::Request, middleware::Next, response::Response};
use std::collections::HashMap;

/// 路由元数据，用于绑定动作和资源模板
#[derive(Clone)]
pub struct RouteMeta {
  /// 动作名，格式 {service}:{verb}
  pub action: &'static str,
  /// 资源模板，支持内置占位符与路由参数占位符
  pub resource_tpl: &'static str,
}

/// 路由元数据注入中间件
/// 为具体端点绑定动作与资源模板
pub async fn inject_route_meta(
  action: &'static str,
  resource_tpl: &'static str,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Response {
  req.extensions_mut().insert(RouteMeta { action, resource_tpl });
  next.run(req).await
}

/// 扩展参数（extras）注入中间件
/// 用于注入路由参数或业务参数
pub async fn inject_extras(
  extras: HashMap<String, String>,
  mut req: Request<axum::body::Body>,
  next: Next,
) -> Response {
  req.extensions_mut().insert(extras);
  next.run(req).await
}

/// 路由元数据宏
/// 简化路由注册时的元数据注入
#[macro_export]
macro_rules! route_with_meta {
  ($router:expr, $method:path, $path:expr, $handler:path, $action:expr, $resource_tpl:expr) => {{
    use crate::web::route_meta::RouteMeta;
    use axum::{http::Request, middleware};

    async fn inject_meta(mut req: Request<axum::body::Body>, next: Next) -> Response {
      req.extensions_mut().insert(RouteMeta { action: $action, resource_tpl: $resource_tpl });
      next.run(req).await
    }

    $router.route($path, $method($handler)).route_layer(middleware::from_fn(inject_meta))
  }};
}

/// 带额外参数的路由宏
#[macro_export]
macro_rules! route_with_meta_and_extras {
  ($router:expr, $method:path, $path:expr, $handler:path, $action:expr, $resource_tpl:expr, $extras:expr) => {{
    use crate::web::route_meta::RouteMeta;
    use axum::{http::Request, middleware};

    async fn inject_meta(mut req: Request<axum::body::Body>, next: Next) -> Response {
      req.extensions_mut().insert(RouteMeta { action: $action, resource_tpl: $resource_tpl });
      next.run(req).await
    }

    async fn inject_extras(mut req: Request<axum::body::Body>, next: Next) -> Response {
      req.extensions_mut().insert($extras);
      next.run(req).await
    }

    $router
      .route($path, $method($handler))
      .route_layer(middleware::from_fn(inject_extras))
      .route_layer(middleware::from_fn(inject_meta))
  }};
}
