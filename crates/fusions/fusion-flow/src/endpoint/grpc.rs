use std::future::Future;

use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use ultimate::{application::Application, configuration::ConfigRegistry};
use ultimate_grpc::{config::GrpcConfig, utils::init_grpc_server, GrpcSettings, GrpcStartInfo};

use crate::service::scheduler_api::flow_api_grpc_svc;

pub async fn grpc_serve(
  app: &Application,
) -> ultimate::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = ultimate::Result<()>>)> {
  let grpc_conf: GrpcConfig = app.get_config()?;

  #[cfg(not(feature = "tonic-reflection"))]
  let encoded_file_descriptor_sets = vec![];
  #[cfg(feature = "tonic-reflection")]
  let encoded_file_descriptor_sets = vec![crate::pb::fusion_flow::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();

  rb.add_service(flow_api_grpc_svc());

  init_grpc_server(GrpcSettings { conf: &grpc_conf, encoded_file_descriptor_sets, routes: rb.routes() }).await
}
