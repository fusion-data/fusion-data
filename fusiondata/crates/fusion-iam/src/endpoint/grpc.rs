use futures::Future;
use ultimate::DataError;
use ultimate_grpc::utils::init_grpc_server;

use crate::{app::get_app_state, auth::auth_svc, permission::permission_svc, role::role_svc, user::grpc::user_svc};

pub fn grpc_serve() -> ultimate::Result<impl Future<Output = std::result::Result<(), DataError>>> {
  let grpc_conf = get_app_state().ultimate_config().grpc();

  #[cfg(not(feature = "tonic-reflection"))]
  let sets = [];
  #[cfg(feature = "tonic-reflection")]
  let sets = [crate::pb::fusion_iam::v1::FILE_DESCRIPTOR_SET];

  init_grpc_server(grpc_conf, sets, |s| {
    s.add_service(permission_svc()).add_service(role_svc()).add_service(user_svc()).add_service(auth_svc());
  })
}
