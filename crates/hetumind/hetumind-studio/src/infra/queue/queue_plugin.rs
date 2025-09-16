use async_trait::async_trait;
use fusion_core::{application::ApplicationBuilder, configuration::ConfigRegistry, plugin::Plugin};

use crate::infra::queue::{QueueConfig, QueueProvider};

pub struct QueueProviderPlugin;

#[async_trait]
impl Plugin for QueueProviderPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let config: QueueConfig = app
      .get_config_by_path("hetumind.queue")
      .expect("QueuePlugin config load failed, please check the config file: `hetumind.queue`");
    let queue = QueueProvider::create(config, app.component()).await.unwrap();
    app.add_component(queue);
  }

  fn dependencies(&self) -> Vec<&str> {
    vec![std::any::type_name::<fusion_db::DbPlugin>()]
  }
}
