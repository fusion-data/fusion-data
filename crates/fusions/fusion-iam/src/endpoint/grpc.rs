use std::future::Future;

use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use ultimate_core::{application::Application, configuration::ConfigRegistry};
use ultimate_grpc::{config::GrpcConfig, utils::init_grpc_server, GrpcSettings, GrpcStartInfo};

use crate::{access_control::AccessControlRpc, auth::AuthRpc, permission::PermissionRpc, role::RoleRpc, user::UserRpc};

pub async fn grpc_serve(
  app: &Application,
) -> ultimate_core::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = ultimate_core::Result<()>>)> {
  let conf: GrpcConfig = app.get_config()?;

  #[cfg(not(feature = "tonic-reflection"))]
  let encoded_file_descriptor_sets = vec![];
  #[cfg(feature = "tonic-reflection")]
  let encoded_file_descriptor_sets = vec![crate::pb::fusion_iam::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();
  rb.add_service(app.component::<AccessControlRpc>().into_rpc())
    .add_service(app.component::<PermissionRpc>().into_rpc())
    .add_service(app.component::<RoleRpc>().into_rpc())
    .add_service(app.component::<UserRpc>().into_rpc())
    .add_service(app.component::<AuthRpc>().into_rpc());

  init_grpc_server(GrpcSettings { conf: &conf, encoded_file_descriptor_sets, routes: rb.routes() }).await
}
