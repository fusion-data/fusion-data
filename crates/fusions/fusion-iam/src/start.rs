use fusiondata_context::app::get_app_state;

use crate::endpoint::grpc::grpc_serve;

pub async fn start_fusion_iam() -> ultimate::Result<()> {
  let app = get_app_state();
  let (_rx, grpc_serve_fut) = grpc_serve(app).await?;
  grpc_serve_fut.await
}
