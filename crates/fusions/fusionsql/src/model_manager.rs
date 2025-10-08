use fusion_common::ctx::Ctx;

use crate::config::DbConfig;
use crate::store::{Dbx, create_dbx};
use crate::{Result, SqlError};

#[derive(Debug, Clone)]
pub struct ModelManager {
  dbx: Dbx,
  ctx: Option<Ctx>,
}

impl ModelManager {
  /// Constructor
  pub async fn new(db_config: &DbConfig, application_name: Option<&str>) -> Result<Self> {
    let dbx = create_dbx(db_config, application_name).await?;
    Ok(ModelManager { dbx, ctx: None })
  }

  /// 返回一个新的事务
  pub fn txn_cloned(&self) -> ModelManager {
    let dbx = self.dbx.txn_cloned();
    ModelManager { dbx, ctx: self.ctx.clone() }
  }

  /// 若当前 ModelManager 已开启事务，则返回self的克隆，否则返回一个新的事务。
  pub fn get_txn_clone(&self) -> ModelManager {
    if self.dbx().is_txn() { self.clone() } else { self.txn_cloned() }
  }

  pub fn dbx(&self) -> &Dbx {
    &self.dbx
  }

  pub fn ctx_ref(&self) -> Result<&Ctx> {
    self.ctx.as_ref().ok_or(SqlError::Unauthorized("The ctx of ModelManager is not set".to_string()))
  }

  pub fn ctx_opt_ref(&self) -> Option<&Ctx> {
    self.ctx.as_ref()
  }

  pub fn with_ctx(mut self, ctx: Ctx) -> Self {
    self.ctx = Some(ctx);
    self
  }
}
