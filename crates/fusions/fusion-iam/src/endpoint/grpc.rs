use std::future::Future;

use fusion_server::app::get_app_state;
use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use ultimate_grpc::{utils::init_grpc_server, GrpcSettings, GrpcStartInfo};

use crate::{
  access_control::access_control_svc, auth::auth_svc, permission::permission_svc, role::role_svc, user::grpc::user_svc,
};

pub async fn grpc_serve(
) -> ultimate::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = ultimate::Result<()>>)> {
  let conf = get_app_state().configuration().grpc();

  #[cfg(not(feature = "tonic-reflection"))]
  let encoded_file_descriptor_sets = vec![];
  #[cfg(feature = "tonic-reflection")]
  let encoded_file_descriptor_sets = vec![crate::pb::fusion_iam::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();
  rb.add_service(access_control_svc())
    .add_service(permission_svc())
    .add_service(role_svc())
    .add_service(user_svc())
    .add_service(auth_svc());

  init_grpc_server(GrpcSettings { conf, encoded_file_descriptor_sets, routes: rb.routes() }).await
}
