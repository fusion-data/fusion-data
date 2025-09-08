use std::net::SocketAddr;

use ::config::{File, FileFormat};
use config::{DEFAULT_CONFIG_STR, GrpcConfig};
use fusion_core::{application::ApplicationBuilder, async_trait, plugin::Plugin};
use tonic::service::{Routes, interceptor::InterceptedService};

pub mod config;
pub mod utils;

pub type GrpcServiceIntercepted<S> =
  InterceptedService<S, fn(tonic::Request<()>) -> core::result::Result<tonic::Request<()>, tonic::Status>>;

#[derive(Debug)]
pub struct GrpcStartInfo {
  pub local_addr: SocketAddr,
}

pub struct GrpcSettings {
  pub conf: GrpcConfig,
  /// tonic generated file descriptor sets
  pub encoded_file_descriptor_sets: Vec<&'static [u8]>,
  pub routes: Routes,
}

pub struct GrpcPlugin;

#[async_trait]
impl Plugin for GrpcPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    app.add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml));
  }
}
