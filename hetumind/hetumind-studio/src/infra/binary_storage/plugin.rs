use async_trait::async_trait;
use fusion_core::{application::ApplicationBuilder, configuration::ConfigRegistry, plugin::Plugin};
use hetumind_core::binary_storage::BinaryStorageConfig;

use crate::binary_storage::BinaryDataManagerFactory;

pub struct BinaryDataManagerPlugin;

#[async_trait]
impl Plugin for BinaryDataManagerPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    let config: BinaryStorageConfig = app
      .get_config_by_path("hetumind.binary_storage")
      .expect("Failed to obtain configuration 'hetumind.binary_storage'");
    let binary_data_manager =
      BinaryDataManagerFactory::create_from_config(config).await.expect("Create BinaryDataManager error");
    app.add_component(binary_data_manager);
  }
}
