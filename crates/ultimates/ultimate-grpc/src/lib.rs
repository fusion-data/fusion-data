use std::net::SocketAddr;

use tonic::service::interceptor::InterceptedService;

pub mod utils;

pub type GrpcServiceIntercepted<S> =
  InterceptedService<S, fn(tonic::Request<()>) -> core::result::Result<tonic::Request<()>, tonic::Status>>;

#[derive(Debug)]
pub struct GrpcStartInfo {
  pub local_addr: SocketAddr,
}
