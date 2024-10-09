use std::{net::SocketAddr, time::Duration};

use futures::{Future, TryFutureExt};
use prost_types::FieldMask;
use tokio::net::TcpListener;
use tonic::{
  metadata::MetadataMap,
  service::Routes,
  transport::{server::TcpIncoming, Server},
  Status,
};
use tower_http::trace::TraceLayer;
use ultimate::{
  configuration::model::{GrpcConf, SecurityConf},
  security::{jose::JwtPayload, SecurityUtils},
  DataError,
};

pub async fn init_grpc_server<'b>(
  conf: &GrpcConf,
  encoded_file_descriptor_sets: impl IntoIterator<Item = &'b [u8]>,
  mut routes: Routes,
) -> ultimate::Result<(SocketAddr, impl Future<Output = std::result::Result<(), DataError>>)> {
  let tcp_listener = TcpListener::bind(&conf.server_addr).await?;
  let local_addr = tcp_listener.local_addr()?;

  let mut b = Server::builder().layer(TraceLayer::new_for_grpc());

  #[cfg(feature = "tonic-web")]
  let mut b = b.accept_http1(true).layer(tonic_web::GrpcWebLayer::new());

  #[cfg(feature = "tonic-reflection")]
  {
    let rb = encoded_file_descriptor_sets
      .into_iter()
      .fold(tonic_reflection::server::Builder::configure(), |rb, set| rb.register_encoded_file_descriptor_set(set));
    let service = rb.build_v1().unwrap();
    routes = routes.add_service(service);
  }

  let tcp_incoming = TcpIncoming::from_listener(tcp_listener, false, Some(Duration::from_secs(30)))
    .map_err(|_| DataError::server_error("Bind tcp listener failed"))?;

  let serve = b.add_routes(routes).serve_with_incoming(tcp_incoming).map_err(DataError::from);
  Ok((local_addr, serve))
}

pub fn extract_jwt_payload_from_metadata(
  sc: &SecurityConf,
  metadata: &MetadataMap,
) -> Result<JwtPayload, tonic::Status> {
  let token = extract_token_from_metadata(metadata)?;
  let (payload, _) =
    SecurityUtils::decrypt_jwt(sc.pwd(), token).map_err(|e| Status::unauthenticated(e.to_string()))?;
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

#[cfg(feature = "uuid")]
pub fn parse_uuid(s: &str) -> core::result::Result<uuid::Uuid, Status> {
  uuid::Uuid::parse_str(s).map_err(|e| Status::invalid_argument(format!("Invalid uuid: {}", e)))
}
