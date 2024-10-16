use std::sync::{Arc, OnceLock};

use ultimate::{
  configuration::{Configuration, ConfigurationState},
  ctx::Ctx,
  starter,
};
use ultimate_db::{DbState, ModelManager};

use crate::ctx::{CtxW, RequestMetadata};

#[derive(Clone)]
pub struct AppState {
  pub configuration_state: ConfigurationState,
  pub db_state: DbState,
}

impl AppState {
  pub fn configuration(&self) -> &Configuration {
    self.configuration_state().configuration()
  }

  pub fn mm(&self) -> &ModelManager {
    self.db_state().mm()
  }

  pub fn create_root_ctx(&self) -> CtxW {
    CtxW::new(self, Ctx::new_root(), Arc::new(RequestMetadata::default()))
  }

  pub fn create_super_admin_ctx(&self) -> CtxW {
    CtxW::new(self, Ctx::new_super_admin(), Arc::new(RequestMetadata::default()))
  }

  pub fn configuration_state(&self) -> &ConfigurationState {
    &self.configuration_state
  }

  pub fn db_state(&self) -> &DbState {
    &self.db_state
  }
}

pub fn get_app_state() -> &'static AppState {
  static APP: OnceLock<AppState> = OnceLock::new();

  APP.get_or_init(|| new_app_state().unwrap())
}

fn new_app_state() -> ultimate::Result<AppState> {
  let configuration_state = starter::load_and_init();
  let db_state = DbState::from_config(configuration_state.configuration().db())?;
  let app = AppState { configuration_state, db_state };
  Ok(app)
}
