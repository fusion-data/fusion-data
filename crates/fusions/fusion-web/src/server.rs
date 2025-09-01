use axum::Router;
use log::info;

use fusion_core::{DataError, application::Application, configuration::ConfigRegistry};

use crate::config::WebConfig;

pub async fn init_server(router: Router) -> fusion_core::Result<()> {
  let app = Application::global();
  let conf: WebConfig = app.get_config()?;
  init_server_with_config(&conf, router).await
}

pub async fn init_server_with_config(conf: &WebConfig, router: Router) -> fusion_core::Result<()> {
  let make_service = router.into_make_service();
  let listener = tokio::net::TcpListener::bind(conf.server_addr()).await.unwrap();
  let sock_addr = listener.local_addr()?;
  info!("The Web Server listening on {}", sock_addr);
  axum::serve(listener, make_service).await.map_err(DataError::from)
}
