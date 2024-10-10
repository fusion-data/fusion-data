use fusion_server::app::get_app_state;
use tokio::sync::oneshot;

use crate::endpoint::grpc::grpc_serve;

pub async fn start_fusion_iam() -> ultimate::Result<()> {
  let _app = get_app_state();
  let (tx, _rx) = oneshot::channel();
  grpc_serve(tx).await
}
