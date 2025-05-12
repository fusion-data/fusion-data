mod api_helpers;
mod macro_helpers;

use ::config::{File, FileFormat};
use async_trait::async_trait;
pub use modelsql::{DbConfig, ModelManager};
use ultimate_core::{application::ApplicationBuilder, configuration::ConfigRegistry, plugin::Plugin};

pub mod acs;
pub use api_helpers::*;

pub const DEFAULT_CONFIG_STR: &str = include_str!("../resources/default.toml");

pub struct DbPlugin;

#[async_trait]
impl Plugin for DbPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    sqlx::any::install_default_drivers();
    app.add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml));
    let config: DbConfig = app.get_config_by_path("ultimate.db").expect("sqlx plugin config load failed");
    let mm = ModelManager::new(&config, Some(app.get_ultimate_config().app().name())).expect("Init db state failed");
    app.add_component(mm);
  }
}
