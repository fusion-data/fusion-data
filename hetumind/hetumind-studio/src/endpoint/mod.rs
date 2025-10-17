use axum::{Router, middleware::from_fn_with_state};
use fusion_core::{DataError, application::Application};
use fusion_web::server::WebServerBuilder;
use http::header::AUTHORIZATION;
use jieyuan_core::web::path_authz::path_authz_middleware;
use tower_http::{
  compression::CompressionLayer,
  cors::{self, CorsLayer},
  sensitive_headers::SetSensitiveRequestHeadersLayer,
  trace::{DefaultMakeSpan, TraceLayer},
};

pub mod api;

pub async fn init_web(app: Application) -> Result<(), DataError> {
  let router = Router::new()
        .nest("/api", api::routes())
        .with_state(app.clone())
        .layer(TraceLayer::new_for_http()
        // .on_response(DefaultOnResponse::new().include_headers(true))
        .make_span_with(DefaultMakeSpan::new().include_headers(true)))
        .layer(CorsLayer::new().allow_methods(cors::Any).allow_origin(cors::Any))
        .layer(SetSensitiveRequestHeadersLayer::new(vec![AUTHORIZATION]))
        .layer(CompressionLayer::new())
        // 只需要添加这一个权限中间件
        .layer(from_fn_with_state(app, path_authz_middleware));

  WebServerBuilder::new(router).build().await
}
