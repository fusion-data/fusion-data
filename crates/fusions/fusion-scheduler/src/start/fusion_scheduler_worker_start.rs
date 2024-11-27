use ultimate::application::Application;

pub async fn fusion_scheduler_worker_start() -> ultimate::Result<()> {
  Application::builder().run().await?;
  Ok(())
}
