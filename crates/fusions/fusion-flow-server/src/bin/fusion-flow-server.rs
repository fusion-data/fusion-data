use fusion_flow::start::FusionFlowStart;
use fusion_flow_server::start::FusionFlowServerStart;
use ultimate_core::{application::Application, timer::TimerPlugin};
use ultimate_grpc::GrpcPlugin;

#[tokio::main]
async fn main() -> ultimate_core::Result<()> {
  let mut app_builder = Application::builder();
  app_builder.add_plugin(TimerPlugin).add_plugin(GrpcPlugin);

  let starter = FusionFlowStart::init(&mut app_builder).await?;
  let server_starter = FusionFlowServerStart::init(&mut app_builder).await?;

  app_builder.run().await?;

  let (h1, h2) = tokio::join!(starter.start(), server_starter.start());
  h1?;
  h2?;
  Ok(())
}
