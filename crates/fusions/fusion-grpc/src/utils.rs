use std::{future::Future, time::Duration};

use fusion_corelib::ctx::CtxPayload;
use futures::TryFutureExt;
use log::info;
use protobuf::well_known_types::field_mask::FieldMask;
use tokio::{net::TcpListener, sync::oneshot};
use tonic::{
  Status,
  metadata::MetadataMap,
  transport::{Server, server::TcpIncoming},
};
use fusion_common::env::set_env;
use fusion_core::{DataError, configuration::SecurityConfig, security::SecurityUtils};

use crate::{GrpcSettings, GrpcStartInfo};

/// 初始化 grpc 服务
///
/// # Returns:
/// - rx: gRPC 服务启动信息
/// - future: gRPC 服务 Future，调用 .await 进行（阻塞）执行循环
///
#[allow(unused_mut)]
pub async fn init_grpc_server(
  setting: GrpcSettings,
) -> fusion_core::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = fusion_core::Result<()>>)> {
  let conf = &setting.conf;
  let (tx, rx) = oneshot::channel();
  let tcp_listener = TcpListener::bind(&conf.server_addr).await?;
  let local_addr = tcp_listener.local_addr()?;
  set_env("ULTIMATE__GRPC__SERVER_ADDR", &local_addr.to_string())
    .unwrap_or_else(|_| panic!("Failed to set ULTIMATE__GRPC__SERVER_ADDR, value: {}", local_addr));
  let start_info = GrpcStartInfo { local_addr };
  match tx.send(start_info) {
    Ok(_) => info!("gRPC server listening to {}", local_addr),
    Err(e) => panic!("Init gRPC server info failed: {:?}", e),
  };

  let mut routes = setting.routes;

  #[cfg(not(feature = "opentelemetry"))]
  let mut server = Server::builder().layer(tower_http::trace::TraceLayer::new_for_grpc());
  #[cfg(feature = "opentelemetry")]
  let mut server = Server::builder();

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

  let tcp_incoming = TcpIncoming::from(tcp_listener)
    .with_keepalive(Some(Duration::from_secs(30)))
    .with_nodelay(Some(false));

  Ok((rx, server.add_routes(routes).serve_with_incoming(tcp_incoming).map_err(DataError::from)))
}

#[allow(clippy::result_large_err)]
pub fn extract_payload_from_metadata(sc: &SecurityConfig, metadata: &MetadataMap) -> Result<CtxPayload, tonic::Status> {
  let token = extract_token_from_metadata(metadata)?;
  let (payload, _) = SecurityUtils::decrypt_jwt(sc.pwd(), token).map_err(|e| Status::unauthenticated(e.to_string()))?;
  Ok(payload)
}

#[allow(clippy::result_large_err)]
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

#[allow(clippy::result_large_err)]
pub fn parse_uuid(s: &str) -> core::result::Result<uuid::Uuid, Status> {
  uuid::Uuid::parse_str(s).map_err(|e| Status::invalid_argument(format!("Invalid uuid: {}", e)))
}
