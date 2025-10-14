use std::net::SocketAddr;

use axum::Router;
use config::{File, FileFormat};
use log::info;

use fusion_core::{DataError, application::Application, configuration::ConfigRegistry};
use mea::shutdown::ShutdownRecv;

use crate::config::{DEFAULT_CONFIG_STR, WebConfig};

pub struct WebServerBuilder {
  router: Router,
  shutdown_rx: Option<ShutdownRecv>,
}

impl WebServerBuilder {
  pub fn new(router: Router) -> Self {
    Self { router, shutdown_rx: None }
  }

  pub fn with_shutdown(mut self, shutdown_rx: ShutdownRecv) -> Self {
    self.shutdown_rx = Some(shutdown_rx);
    self
  }

  pub async fn build(self) -> Result<(), DataError> {
    let app = Application::global();
    app.config_registry().add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml))?;
    let conf: WebConfig = app.get_config()?;
    self.init_server_with_config(conf).await
  }

  async fn init_server_with_config(self, conf: WebConfig) -> Result<(), DataError> {
    let listener = tokio::net::TcpListener::bind(&conf.server_addr).await.unwrap();
    let local_addr = listener.local_addr()?;
    info!("The Web Server listening on {}", local_addr);
    let serve = if conf.enable_remote_addr {
      let s = axum::serve(listener, self.router.into_make_service_with_connect_info::<SocketAddr>());
      if let Some(shutdown_rx) = self.shutdown_rx {
        s.with_graceful_shutdown(shutdown_rx.is_shutdown_owned()).await
      } else {
        s.await
      }
    } else {
      let s = axum::serve(listener, self.router.into_make_service());
      if let Some(shutdown_rx) = self.shutdown_rx {
        s.with_graceful_shutdown(shutdown_rx.is_shutdown_owned()).await
      } else {
        s.await
      }
    };
    serve.map_err(DataError::from)
  }
}
