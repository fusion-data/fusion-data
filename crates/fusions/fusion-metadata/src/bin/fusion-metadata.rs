use ultimate::application::Application;

#[tokio::main]
async fn main() -> ultimate::Result<()> {
  Application::builder().run().await;

  Ok(())
}
