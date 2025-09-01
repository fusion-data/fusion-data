use fusion_core::application::Application;
use fusion_db::DbPlugin;
use fusion_grpc::GrpcPlugin;

use crate::endpoint::grpc::grpc_serve;

pub async fn start_fusion_iam() -> fusion_core::Result<()> {
  Application::builder().add_plugin(DbPlugin).add_plugin(GrpcPlugin).run().await?;
  let app = Application::global();

  let (_rx, grpc_serve_fut) = grpc_serve(&app).await?;
  grpc_serve_fut.await
}
