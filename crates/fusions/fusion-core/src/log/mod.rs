mod utils;

use async_trait::async_trait;
pub use utils::{get_trace_id, init_log};

use crate::{application::ApplicationBuilder, plugin::Plugin};

pub struct LogPlugin;

#[async_trait]
impl Plugin for LogPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let setting = app.get_fusion_config();
    init_log(setting.log());
  }
}
