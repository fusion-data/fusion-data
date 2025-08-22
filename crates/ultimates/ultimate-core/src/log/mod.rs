mod utils;

use async_trait::async_trait;
pub use utils::*;

use crate::{application::ApplicationBuilder, plugin::Plugin};

pub struct TracingPlugin;

#[async_trait]
impl Plugin for TracingPlugin {
  async fn build(&self, _app: &mut ApplicationBuilder) {
    // TODO:
  }
}
