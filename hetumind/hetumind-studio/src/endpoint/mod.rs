use axum::Router;
use fusion_core::{DataError, application::Application};
use fusion_web::server::WebServerBuilder;
use http::header::AUTHORIZATION;
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
    .with_state(app)
    .layer(TraceLayer::new_for_http()
    // .on_response(DefaultOnResponse::new().include_headers(true))
    .make_span_with(DefaultMakeSpan::new().include_headers(true)))
    .layer(CorsLayer::new().allow_methods(cors::Any).allow_origin(cors::Any))
    .layer(SetSensitiveRequestHeadersLayer::new(vec![AUTHORIZATION]))
    .layer(CompressionLayer::new());

  WebServerBuilder::new(router).build().await
}
