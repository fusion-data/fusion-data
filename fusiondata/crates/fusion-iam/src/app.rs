use std::sync::{Arc, OnceLock};

use derive_getters::Getters;
use typed_builder::TypedBuilder;
use ultimate::{
  configuration::{ConfigState, UltimateConfig},
  ctx::Ctx,
  starter,
};
use ultimate_db::{DbState, ModelManager};

use crate::ctx::{CtxW, RequestMetadata};

#[derive(Clone, TypedBuilder, Getters)]
pub struct AppState {
  pub config_state: ConfigState,
  pub db_state: DbState,
  pub runtime: Arc<tokio::runtime::Runtime>,
}

impl AppState {
  pub fn ultimate_config(&self) -> &UltimateConfig {
    self.config_state().ultimate_config()
  }

  pub fn mm(&self) -> &ModelManager {
    self.db_state().mm()
  }

  pub fn create_root_ctx(&self) -> crate::ctx::CtxW {
    CtxW::new(self, Ctx::new_root(), Arc::new(RequestMetadata::default()))
  }

  pub fn create_super_admin_ctx(&self) -> crate::ctx::CtxW {
    CtxW::new(self, Ctx::new_super_admin(), Arc::new(RequestMetadata::default()))
  }
}

pub fn get_app_state() -> &'static AppState {
  static APP: OnceLock<AppState> = OnceLock::new();

  APP.get_or_init(|| new_app_state().unwrap())
}

fn new_app_state() -> ultimate::Result<AppState> {
  let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
  let config = starter::load_and_init();
  let db = runtime.block_on(DbState::from_config(config.ultimate_config().db()))?;
  let app = AppState::builder().config_state(config).db_state(db).runtime(Arc::new(runtime)).build();
  Ok(app)
}
