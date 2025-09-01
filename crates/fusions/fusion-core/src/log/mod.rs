mod utils;

use async_trait::async_trait;
pub use utils::{get_trace_id, init_log};

use crate::{
  application::ApplicationBuilder,
  configuration::{ConfigRegistry, LogConfig},
  plugin::Plugin,
};

pub struct LogPlugin;

#[async_trait]
impl Plugin for LogPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let log_conf: LogConfig = app.get_config_by_path("fusion.log").unwrap();
    init_log(&log_conf);
  }
}
