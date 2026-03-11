use ::config::{File, FileFormat};
use hetu_common::ctx::Ctx;
use hetu_core::{application::ApplicationBuilder, async_trait, configuration::ConfigRegistry, plugin::Plugin};
pub use hetusql::{DbConfig, ModelManager};

pub mod acs;

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");

pub struct DbPlugin;

#[async_trait]
impl Plugin for DbPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    // sqlx::any::install_default_drivers();
    app.add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml));
    let config: DbConfig = app
      .get_config_by_path("hetu.db")
      .expect("DbPlugin config load failed, please check the config file: `hetu.db`");
    let mm = ModelManager::new(&config, Some(app.get_hetu_config().app().name()))
      .await
      .unwrap()
      .with_ctx(Ctx::new_super_admin());
    app.add_component(mm);
  }
}
