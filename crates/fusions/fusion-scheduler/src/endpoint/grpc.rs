use fusion_server::app::AppState;
use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use ultimate_grpc::{utils::init_grpc_server, GrpcStartInfo};

use crate::service::scheduler_api::scheduler_api_grpc_svc;

pub async fn grpc_serve(app_state: &AppState, tx: oneshot::Sender<GrpcStartInfo>) -> ultimate::Result<()> {
  let grpc_conf = app_state.configuration().grpc();

  #[cfg(not(feature = "tonic-reflection"))]
  let file_descriptor_sets = [];
  #[cfg(feature = "tonic-reflection")]
  let file_descriptor_sets = [crate::pb::fusion_scheduler::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();

  rb.add_service(scheduler_api_grpc_svc());

  init_grpc_server(grpc_conf, file_descriptor_sets, rb.routes(), tx).await
}
