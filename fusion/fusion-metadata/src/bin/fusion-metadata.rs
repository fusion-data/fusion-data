use ultimate_core::application::Application;

#[tokio::main]
async fn main() -> ultimate_core::Result<()> {
  Application::builder().run().await?;

  Ok(())
}
