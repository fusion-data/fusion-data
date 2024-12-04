use ultimate::ctx::Ctx;

use crate::config::DbConfig;
use crate::store::{dbx::new_db_pool_from_config, Dbx};

use crate::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
  dbx: Dbx,
  ctx: Option<Ctx>,
}

impl ModelManager {
  /// Constructor
  pub fn new(db_config: &DbConfig, application_name: Option<&str>) -> Result<Self> {
    let db_pool = new_db_pool_from_config(db_config, application_name)
      .map_err(|ex| Error::CantCreateModelManagerProvider(ex.to_string()))?;
    let dbx = Dbx::new(db_pool, false);
    Ok(ModelManager { dbx, ctx: None })
  }

  /// 返回一个新的事务
  pub fn txn_cloned(&self) -> ModelManager {
    let dbx = Dbx::new(self.dbx.db().clone(), true);
    ModelManager { dbx, ctx: self.ctx.clone() }
  }

  /// 若当前 ModelManager 已开启事务，则返回self的克隆，否则返回一个新的事务。
  pub fn get_txn_clone(&self) -> ModelManager {
    if self.dbx().is_txn() {
      self.clone()
    } else {
      self.txn_cloned()
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
