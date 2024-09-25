use std::sync::Arc;

use axum::Router;
use tower_http::{
  compression::CompressionLayer,
  cors::{self, CorsLayer},
  trace::TraceLayer,
};
use tracing::info;

use ultimate::configuration::Configuration;

pub async fn init_server(conf: Arc<Configuration>, app: Router) -> ultimate::Result<()> {
  let make_service = app
    .layer(CompressionLayer::new())
    .layer(CorsLayer::new().allow_methods(cors::Any).allow_origin(cors::Any))
    .layer(TraceLayer::new_for_http())
    .into_make_service();

  let listener = tokio::net::TcpListener::bind(conf.web().server_addr()).await.unwrap();
  let sock_addr = listener.local_addr()?;
  info!("The Web Server listening on {}", sock_addr);

  axum::serve(listener, make_service).await?;
  Ok(())
}
