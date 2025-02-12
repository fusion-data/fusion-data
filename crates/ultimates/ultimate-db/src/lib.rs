use ::config::{File, FileFormat};
use async_trait::async_trait;
use config::{DbConfig, DEFAULT_CONFIG_STR};
use ultimate::{application::ApplicationBuilder, configuration::ConfigRegistry, plugin::Plugin};

pub mod acs;
mod api_helpers;
pub mod base;
pub mod config;
mod error;
mod id;
mod macro_helpers;
mod model_manager;
pub mod modql;
mod modql_utils;
pub mod store;

pub use api_helpers::*;
pub use error::{Error, Result};
pub use id::*;
pub use model_manager::*;
pub use modql_utils::*;

pub struct DbPlugin;

#[async_trait]
impl Plugin for DbPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    sqlx::any::install_default_drivers();
    app.add_config_source(File::from_str(DEFAULT_CONFIG_STR, FileFormat::Toml));
    let config: DbConfig = app.get_config().expect("sqlx plugin config load failed");
    let mm = ModelManager::new(&config, Some(app.get_ultimate_config().app().name())).expect("Init db state failed");
    app.add_component(mm);
  }
}
