mod init_tracing;
mod utils;

use async_trait::async_trait;
pub(crate) use init_tracing::*;
pub use utils::*;

use crate::{application::ApplicationBuilder, plugin::Plugin};

pub struct TracingPlugin;

#[async_trait]
impl Plugin for TracingPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    init_tracing(app.get_ultimate_config());
  }
}
