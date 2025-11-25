mod init_tracing;
mod utils;

use std::sync::Arc;

use async_trait::async_trait;
pub(crate) use init_tracing::*;
use init_tracing_opentelemetry::Guard;
pub use utils::*;

use crate::{application::ApplicationBuilder, plugin::Plugin};

#[allow(unused)]
struct GuardMaybe(Option<Guard>);

pub struct TracingPlugin;

#[async_trait]
impl Plugin for TracingPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let guard = init_subscribers(&app.get_fusion_config()).unwrap();

    app.add_component(Arc::new(GuardMaybe(guard)));
  }
}
