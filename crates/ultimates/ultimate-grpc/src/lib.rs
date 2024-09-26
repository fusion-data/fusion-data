use tonic::service::interceptor::InterceptedService;

pub mod utils;

pub type GrpcServiceIntercepted<S> =
  InterceptedService<S, fn(tonic::Request<()>) -> core::result::Result<tonic::Request<()>, tonic::Status>>;
