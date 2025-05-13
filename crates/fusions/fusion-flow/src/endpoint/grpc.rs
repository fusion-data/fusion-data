use std::future::Future;

use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use ultimate_core::{application::Application, configuration::ConfigRegistry};
use ultimate_grpc::{GrpcSettings, GrpcStartInfo, config::GrpcConfig, utils::init_grpc_server};

use crate::service::scheduler_api::flow_api_grpc_svc;

pub async fn grpc_serve(
  app: Application,
) -> ultimate_core::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = ultimate_core::Result<()>>)> {
  let conf: GrpcConfig = app.get_config()?;

  #[cfg(not(feature = "tonic-reflection"))]
  let encoded_file_descriptor_sets: Vec<&'static [u8]> = vec![];
  #[cfg(feature = "tonic-reflection")]
  let encoded_file_descriptor_sets: Vec<&'static [u8]> = vec![crate::pb::fusion_flow::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();

  rb.add_service(flow_api_grpc_svc());

  init_grpc_server(GrpcSettings { conf, encoded_file_descriptor_sets, routes: rb.routes() }).await
}
