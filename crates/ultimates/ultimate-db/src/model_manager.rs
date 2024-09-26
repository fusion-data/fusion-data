use ultimate::{configuration::model::DbConf, ctx::Ctx};

use crate::store::{dbx::new_db_pool_from_config, Dbx};

use crate::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
  dbx: Dbx,
  ctx: Option<Ctx>,
}

impl ModelManager {
  /// Constructor
  pub fn new(db_config: &DbConf) -> Result<Self> {
    let db_pool =
      new_db_pool_from_config(db_config).map_err(|ex| Error::CantCreateModelManagerProvider(ex.to_string()))?;
    let dbx = Dbx::new(db_pool, false)?;
    Ok(ModelManager { dbx, ctx: None })
  }

  pub fn clone_with_txn(&self) -> Result<ModelManager> {
    let dbx = Dbx::new(self.dbx.db().clone(), true)?;
    Ok(ModelManager { dbx, ctx: self.ctx.clone() })
  }

  pub fn get_or_clone_with_txn(&self) -> Result<ModelManager> {
    if self.dbx().is_txn() {
      Ok(self.clone())
    } else {
      Ok(self.clone_with_txn()?)
    }
  }

  pub fn dbx(&self) -> &Dbx {
    &self.dbx
  }

  pub fn ctx_ref(&self) -> Result<&Ctx> {
    self.ctx.as_ref().ok_or(Error::Unauthorized)
  }

  pub fn ctx_opt_ref(&self) -> Option<&Ctx> {
    self.ctx.as_ref()
  }

  pub fn with_ctx(mut self, ctx: Ctx) -> Self {
    self.ctx = Some(ctx);
    self
  }
}
