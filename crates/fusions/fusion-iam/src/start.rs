use fusion_server::app::get_app_state;

use crate::endpoint::grpc::grpc_serve;

pub async fn start_fusion_iam() -> ultimate::Result<()> {
  let _app = get_app_state();
  grpc_serve().await?.1.await?;

  Ok(())
}
