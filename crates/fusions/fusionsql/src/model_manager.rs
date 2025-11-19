use std::sync::Arc;

use fusion_common::ctx::Ctx;
use fusionsql_core::filter::FilterGroups;

use crate::base::BmcConfig;
use crate::config::DbConfig;
use crate::store::{Dbx, create_dbx};
use crate::{Result, SqlError};

/// 过滤器拦截器类型
pub type FilterInterceptor = Arc<dyn Fn(&BmcConfig, Option<&Ctx>, FilterGroups) -> Result<FilterGroups> + Send + Sync>;

#[derive(Clone)]
pub struct ModelManager {
  dbx: Dbx,
  ctx: Option<Ctx>,
  filter_interceptor: Option<FilterInterceptor>,
}

impl ModelManager {
  /// Constructor
  pub async fn new(db_config: &DbConfig, application_name: Option<&str>) -> Result<Self> {
    let dbx = create_dbx(db_config, application_name).await?;
    Ok(ModelManager { dbx, ctx: None, filter_interceptor: None })
  }

  /// 返回一个新的事务
  pub fn txn_cloned(&self) -> ModelManager {
    let dbx = self.dbx.txn_cloned();
    ModelManager { dbx, ctx: self.ctx.clone(), filter_interceptor: self.filter_interceptor.clone() }
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

  /// 设置过滤器拦截器
  /// 拦截器函数会在查询执行前被调用，可以修改过滤条件
  pub fn with_filter_interceptor<F>(mut self, interceptor: F) -> Self
  where
    F: Fn(&BmcConfig, Option<&Ctx>, FilterGroups) -> Result<FilterGroups> + Send + Sync + 'static,
  {
    self.filter_interceptor = Some(Arc::new(interceptor));
    self
  }

  /// 应用过滤器拦截器（如果存在）
  pub fn apply_filter_interceptor(&self, bmc_config: &BmcConfig, filters: FilterGroups) -> Result<FilterGroups> {
    if let Some(interceptor) = self.filter_interceptor.as_ref() {
      interceptor(bmc_config, self.ctx_opt_ref(), filters)
    } else {
      Ok(filters)
    }
  }

  /// 应用过滤器拦截器（可选CTX版本）
  pub fn apply_filter_interceptor_with_ctx(
    &self,
    bmc_config: &BmcConfig,
    ctx: &Ctx,
    filters: FilterGroups,
  ) -> Result<FilterGroups> {
    if let Some(interceptor) = &self.filter_interceptor {
      interceptor(bmc_config, Some(ctx), filters)
    } else {
      Ok(filters)
    }
  }
}
