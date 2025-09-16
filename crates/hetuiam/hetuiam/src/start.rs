use fusion_core::application::Application;
use fusion_db::DbPlugin;

pub async fn start_fusion_iam() -> fusion_core::Result<()> {
  Application::builder().add_plugin(DbPlugin).run().await?;
  let app = Application::global();

  // TODO start web

  Ok(())
}
