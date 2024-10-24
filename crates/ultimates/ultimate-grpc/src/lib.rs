use std::net::SocketAddr;

use tonic::service::{interceptor::InterceptedService, Routes};
use ultimate::configuration::model::GrpcConfig;

pub mod utils;

pub type GrpcServiceIntercepted<S> =
  InterceptedService<S, fn(tonic::Request<()>) -> core::result::Result<tonic::Request<()>, tonic::Status>>;

#[derive(Debug)]
pub struct GrpcStartInfo {
  pub local_addr: SocketAddr,
}

pub struct GrpcSettings<'b> {
  pub conf: &'b GrpcConfig,
  /// tonic generated file descriptor sets
  pub encoded_file_descriptor_sets: Vec<&'b [u8]>,
  pub routes: Routes,
}
