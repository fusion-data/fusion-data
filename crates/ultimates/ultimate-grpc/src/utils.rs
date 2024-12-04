use std::{future::Future, time::Duration};

use futures::TryFutureExt;
use prost_types::FieldMask;
use tokio::{net::TcpListener, sync::oneshot};
use tonic::{
  metadata::MetadataMap,
  transport::{server::TcpIncoming, Server},
  Status,
};
use tracing::info;
use ultimate::{
  configuration::SecurityConfig,
  security::{jose::JwtPayload, SecurityUtils},
  DataError,
};

use crate::{GrpcSettings, GrpcStartInfo};

/// 初始化 grpc 服务
///
/// # Returns:
/// - rx: gRPC 服务启动信息
/// - future: gRPC 服务 Future，调用 .await 进行（阻塞）执行循环
///
#[allow(unused_mut)]
pub async fn init_grpc_server(
  setting: GrpcSettings<'_>,
) -> ultimate::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = ultimate::Result<()>>)> {
  let conf = &setting.conf;
  let (tx, rx) = oneshot::channel();
  let tcp_listener = TcpListener::bind(&conf.server_addr).await?;
  let local_addr = tcp_listener.local_addr()?;
  std::env::set_var("ULTIMATE__GRPC__SERVER_ADDR", local_addr.to_string());
  let start_info = GrpcStartInfo { local_addr };
  match tx.send(start_info) {
    Ok(_) => info!("gRPC server listening to {}", local_addr),
    Err(e) => panic!("Init gRPC server info failed: {:?}", e),
  };

  let mut routes = setting.routes;

  #[cfg(not(feature = "opentelemetry"))]
  let mut server = Server::builder().layer(tower_http::trace::TraceLayer::new_for_grpc());
  #[cfg(feature = "opentelemetry")]
  let mut server = Server::builder().layer(
    tonic_tracing_opentelemetry::middleware::server::OtelGrpcLayer::default()
      .filter(tonic_tracing_opentelemetry::middleware::filters::reject_healthcheck),
  );

  #[cfg(feature = "tonic-web")]
  let mut server = server.accept_http1(true).layer(tonic_web::GrpcWebLayer::new());

  #[cfg(feature = "tonic-reflection")]
  {
    let rb = setting
      .encoded_file_descriptor_sets
      .into_iter()
      .fold(tonic_reflection::server::Builder::configure(), |rb, set| rb.register_encoded_file_descriptor_set(set));
    let service = rb.build_v1().unwrap();
    routes = routes.add_service(service);
  }

  let tcp_incoming = TcpIncoming::from_listener(tcp_listener, false, Some(Duration::from_secs(30)))
    .map_err(|_| DataError::server_error("Bind tcp listener failed"))?;

  Ok((rx, server.add_routes(routes).serve_with_incoming(tcp_incoming).map_err(DataError::from)))
}

pub fn extract_jwt_payload_from_metadata(
  sc: &SecurityConfig,
  metadata: &MetadataMap,
) -> Result<JwtPayload, tonic::Status> {
  let token = extract_token_from_metadata(metadata)?;
  let (payload, _) = SecurityUtils::decrypt_jwt(sc.pwd(), token).map_err(|e| Status::unauthenticated(e.to_string()))?;
  Ok(payload)
}

pub fn extract_token_from_metadata(metadata: &MetadataMap) -> Result<&str, tonic::Status> {
  let auth_header = metadata
    .get("authorization")
    .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;
  let auth_str = auth_header.to_str().map_err(|_| Status::unauthenticated("Invalid authorization header"))?;
  let offset = if auth_str.starts_with("Bearer ") { 7 } else { 0 };

  Ok(&auth_str[offset..])
}

// 当 paths 为空或者 paths 包含以 path 开头的路径时返回 true，否则返回 false
pub fn field_mask_match_with(field_mask: &FieldMask, path: &str) -> bool {
  field_mask.paths.is_empty() || field_mask.paths.iter().any(|p| p.starts_with(path))
}

pub fn parse_uuid(s: &str) -> core::result::Result<uuid::Uuid, Status> {
  uuid::Uuid::parse_str(s).map_err(|e| Status::invalid_argument(format!("Invalid uuid: {}", e)))
}
