use std::future::Future;

use tokio::sync::oneshot;
use tonic::service::RoutesBuilder;
use fusion_core::{application::Application, configuration::ConfigRegistry};
use fusion_grpc::{GrpcSettings, GrpcStartInfo, config::GrpcConfig, utils::init_grpc_server};

use crate::{access_control::AccessControlRpc, auth::AuthRpc, permission::PermissionRpc, role::RoleRpc, user::UserRpc};

pub async fn grpc_serve(
  app: &Application,
) -> fusion_core::Result<(oneshot::Receiver<GrpcStartInfo>, impl Future<Output = fusion_core::Result<()>>)> {
  let conf: GrpcConfig = app.get_config()?;

  #[cfg(not(feature = "tonic-reflection"))]
  let encoded_file_descriptor_sets: Vec<&'static [u8]> = vec![];
  #[cfg(feature = "tonic-reflection")]
  let encoded_file_descriptor_sets: Vec<&'static [u8]> = vec![crate::pb::fusion_iam::v1::FILE_DESCRIPTOR_SET];

  let mut rb = RoutesBuilder::default();
  rb.add_service(app.component::<AccessControlRpc>().into_rpc())
    .add_service(app.component::<PermissionRpc>().into_rpc())
    .add_service(app.component::<RoleRpc>().into_rpc())
    .add_service(app.component::<UserRpc>().into_rpc())
    .add_service(app.component::<AuthRpc>().into_rpc());

  init_grpc_server(GrpcSettings { conf, encoded_file_descriptor_sets, routes: rb.routes() }).await
}
