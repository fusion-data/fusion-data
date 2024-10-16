use std::future::Future;

use fusion_server::app::AppState;
use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use ultimate_grpc::{utils::init_grpc_server, GrpcSettings, GrpcStartInfo};

use crate::service::scheduler_api::scheduler_api_grpc_svc;

pub async fn grpc_serve(
  app_state: &AppState,
) -> ultimate::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = ultimate::Result<()>>)> {
  let grpc_conf = app_state.configuration().grpc();

  #[cfg(not(feature = "tonic-reflection"))]
  let encoded_file_descriptor_sets = vec![];
  #[cfg(feature = "tonic-reflection")]
  let encoded_file_descriptor_sets = vec![crate::pb::fusion_scheduler::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();

  rb.add_service(scheduler_api_grpc_svc());

  init_grpc_server(GrpcSettings { conf: grpc_conf, encoded_file_descriptor_sets, routes: rb.routes() }).await
}
