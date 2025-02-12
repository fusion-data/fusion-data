use ultimate::{
  application::{Application, ApplicationBuilder},
  utils::handle_join_error,
};

use crate::endpoint::grpc_serve;

pub struct FusionFlowStart {}

impl FusionFlowStart {
  pub async fn init(_app_builder: &mut ApplicationBuilder) -> ultimate::Result<Self> {
    Ok(Self {})
  }

  pub async fn start(self) -> ultimate::Result<()> {
    let app: Application = Application::global();

    let (_rx, grpc_serve_fut) = grpc_serve(&app).await?;
    let grpc_serve_handle = tokio::spawn(grpc_serve_fut);

    let (grpc_ret,) = tokio::join!(grpc_serve_handle);

    handle_join_error(grpc_ret, "grpc serve");
    Ok(())
  }
}
