mod execution_store_pg;

use std::sync::Arc;

use async_trait::async_trait;
pub use execution_store_pg::ExecutionStorePg;
use fusion_core::{application::ApplicationBuilder, plugin::Plugin};

use crate::runtime::execution::ExecutionStore;

pub type ExecutionStoreService = Arc<dyn ExecutionStore>;

pub struct ExecutionStorePlugin;

#[async_trait]
impl Plugin for ExecutionStorePlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let execution_store: ExecutionStoreService = Arc::new(ExecutionStorePg::new(app.component()));
    app.add_component(execution_store);
  }

  fn dependencies(&self) -> Vec<&str> {
    vec![std::any::type_name::<fusion_db::DbPlugin>()]
  }
}
