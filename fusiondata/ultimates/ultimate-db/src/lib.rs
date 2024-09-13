use ultimate::configuration::model::DbConf;

pub mod acs;
pub mod base;
mod error;
mod id;
mod model_manager;
mod modql_utils;
pub mod store;

pub use error::{Error, Result};
pub use id::*;
pub use model_manager::*;
pub use modql_utils::*;

#[derive(Clone)]
pub struct DbState {
  mm: ModelManager,
}

impl DbState {
  pub async fn from_config(db: &DbConf) -> Result<Self> {
    let mm = ModelManager::new(db).await?;
    Ok(DbState { mm })
  }

  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }
}
