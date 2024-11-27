use ultimate::application::Application;
use ultimate_db::DbPlugin;
use ultimate_grpc::GrpcPlugin;

use crate::endpoint::grpc::grpc_serve;

pub async fn start_fusion_iam() -> ultimate::Result<()> {
  Application::builder().add_plugin(DbPlugin).add_plugin(GrpcPlugin).run().await?;
  let app = Application::global();

  let (_rx, grpc_serve_fut) = grpc_serve(&app).await?;
  grpc_serve_fut.await
}
