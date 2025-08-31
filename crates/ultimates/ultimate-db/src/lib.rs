use ::config::{File, FileFormat};
use fusion_corelib::ctx::Ctx;
pub use modelsql::{DbConfig, ModelManager};
use ultimate_core::{application::ApplicationBuilder, async_trait, configuration::ConfigRegistry, plugin::Plugin};

pub mod acs;

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");

pub struct DbPlugin;

#[async_trait]
impl Plugin for DbPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    // sqlx::any::install_default_drivers();
    app.add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml));
    let config: DbConfig = app
      .get_config_by_path("ultimate.db")
      .expect("DbPlugin config load failed, please check the config file: `ultimate.db`");
    let mm = ModelManager::new(&config, Some(app.get_ultimate_config().app().name()))
      .await
      .unwrap()
      .with_ctx(Ctx::new_super_admin());
    app.add_component(mm);
  }
}
