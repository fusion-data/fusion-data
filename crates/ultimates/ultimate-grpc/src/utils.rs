use futures::{Future, TryFutureExt};
use prost_types::FieldMask;
use tonic::{metadata::MetadataMap, service::RoutesBuilder, transport::Server, Status};
use tower_http::trace::TraceLayer;
use ultimate::{
  configuration::model::{GrpcConf, SecurityConf},
  security::{jose::JwtPayload, SecurityUtils},
  DataError,
};

pub fn init_grpc_server<'b, F>(
  conf: &GrpcConf,
  _encoded_file_descriptor_sets: impl IntoIterator<Item = &'b [u8]>,
  f: F,
) -> ultimate::Result<impl Future<Output = std::result::Result<(), DataError>>>
where
  F: FnOnce(&mut RoutesBuilder),
{
  let grpc_addr = conf.server_addr.parse()?;

  #[allow(unused_mut)]
  let mut b = Server::builder().layer(TraceLayer::new_for_grpc());

  #[cfg(feature = "tonic-web")]
  let mut b = b.accept_http1(true).layer(tonic_web::GrpcWebLayer::new());

  let mut routes_builder = RoutesBuilder::default();
  f(&mut routes_builder);

  #[cfg(feature = "tonic-reflection")]
  {
    let rb = _encoded_file_descriptor_sets
      .into_iter()
      .fold(tonic_reflection::server::Builder::configure(), |rb, set| rb.register_encoded_file_descriptor_set(set));
    let service = rb.build_v1().unwrap();
    routes_builder.add_service(service);
  }

  // let s = router.into_service();

  let serve = b.add_routes(routes_builder.routes()).serve(grpc_addr).map_err(DataError::from);
  Ok(serve)
}

pub fn extract_jwt_payload_from_metadata(
  sc: &SecurityConf,
  metadata: &MetadataMap,
) -> Result<JwtPayload, tonic::Status> {
  let token = extract_token_from_metadata(metadata)?;
  let (payload, _) = SecurityUtils::decrypt_jwt(sc.pwd(), token).map_err(|e| Status::unauthenticated(e.to_string()))?;
  Ok(payload)
}

pub fn extract_token_from_metadata(metadata: &MetadataMap) -> Result<&str, tonic::Status> {
  let auth_header =
    metadata.get("authorization").ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;
  let auth_str = auth_header.to_str().map_err(|_| Status::unauthenticated("Invalid authorization header"))?;
  let offset = if auth_str.starts_with("Bearer ") { 7 } else { 0 };

  Ok(&auth_str[offset..])
}

// 当 paths 为空或者 paths 包含以 path 开头的路径时返回 true，否则返回 false
pub fn field_mask_match_with(field_mask: &FieldMask, path: &str) -> bool {
  field_mask.paths.is_empty() || field_mask.paths.iter().any(|p| p.starts_with(path))
}
