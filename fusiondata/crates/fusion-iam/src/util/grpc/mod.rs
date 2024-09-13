pub mod interceptor;

use tonic::{metadata::MetadataMap, service::interceptor::InterceptedService, Extensions};

use crate::ctx::CtxW;

pub type GrpcServiceIntercepted<S> =
  InterceptedService<S, fn(tonic::Request<()>) -> core::result::Result<tonic::Request<()>, tonic::Status>>;

pub fn extract_req_parts<T>(request: tonic::Request<T>) -> Result<(MetadataMap, Extensions, T, CtxW), tonic::Status> {
  let (meta, exts, request) = request.into_parts();
  let ctx = (&meta).try_into()?;
  Ok((meta, exts, request, ctx))
}
