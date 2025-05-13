use ultimate_core::application::ApplicationBuilder;
use ultimate_db::DbPlugin;

pub struct FusionFlowServerStart {}

impl FusionFlowServerStart {
  pub async fn init(app_builder: &mut ApplicationBuilder) -> ultimate_core::Result<Self> {
    app_builder.add_plugin(DbPlugin);
    Ok(Self {})
  }

  pub async fn start(self) -> ultimate_core::Result<()> {
    Ok(())
  }
}
