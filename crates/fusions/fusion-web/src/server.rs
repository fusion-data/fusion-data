use std::net::SocketAddr;

use axum::Router;
use config::{File, FileFormat};
use log::info;

use fusion_core::{DataError, application::Application, configuration::ConfigRegistry};

use crate::config::{DEFAULT_CONFIG_STR, WebConfig};

pub async fn init_server(router: Router) -> Result<(), DataError> {
  let app = Application::global();
  app.config_registry().add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml))?;
  let conf: WebConfig = app.get_config()?;
  init_server_with_config(&conf, router).await
}

pub async fn init_server_with_config(conf: &WebConfig, router: Router) -> Result<(), DataError> {
  let listener = tokio::net::TcpListener::bind(&conf.server_addr).await.unwrap();
  let local_addr = listener.local_addr()?;
  info!("The Web Server listening on {}", local_addr);
  let serve = if conf.enable_remote_addr {
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await
  } else {
    axum::serve(listener, router.into_make_service()).await
  };
  serve.map_err(DataError::from)
}
