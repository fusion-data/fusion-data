use fusion_server::app::get_app_state;

use crate::endpoint::grpc_serve;

pub async fn fusion_scheduler_start() -> ultimate::Result<()> {
  let app = get_app_state();
  grpc_serve(app)?.await
}
