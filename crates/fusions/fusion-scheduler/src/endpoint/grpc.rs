use std::future::Future;

use fusion_server::app::AppState;
use tonic::service::RoutesBuilder;
use ultimate::DataError;
use ultimate_grpc::utils::init_grpc_server;

use crate::service::scheduler::scheduler_grpc_svc;
pub fn grpc_serve(app_state: &AppState) -> ultimate::Result<impl Future<Output = std::result::Result<(), DataError>>> {
  let grpc_conf = app_state.configuration().grpc();

  #[cfg(not(feature = "tonic-reflection"))]
  let file_descriptor_sets = [];
  #[cfg(feature = "tonic-reflection")]
  let file_descriptor_sets = [crate::pb::fusion_scheduler::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();

  rb.add_service(scheduler_grpc_svc());

  init_grpc_server(grpc_conf, file_descriptor_sets, rb.routes())
}
