use std::future::Future;
use std::sync::Arc;

use hetu_common::ctx::Ctx;
use hetusql_core::filter::FilterGroups;

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
    Ok(Self { dbx, ctx: None, filter_interceptor: None })
  }

  pub fn new_with_dbx(dbx: Dbx) -> Self {
    Self { dbx, ctx: None, filter_interceptor: None }
  }

  /// 强制返回一个新的事务，即使当前 ModelManager 已开启事务。
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

  /// 闭包式事务 API
  ///
  /// 自动管理事务的生命周期，在闭包执行成功时提交事务，失败时回滚事务。
  ///
  /// # 示例
  ///
  /// ```rust
  /// use hetusql::ModelManager;
  ///
  /// async fn example(mm: &ModelManager) -> Result<()> {
  ///   mm.transaction(|mm| async move {
  ///     // 在事务中执行多个操作
  ///     UserBmc::create(&mm, user).await?;
  ///     UserBmc::update(&mm, id, update).await?;
  ///     Ok(())
  ///   }).await?;
  ///   Ok(())
  /// }
  /// ```
  ///
  /// # 嵌套事务支持
  ///
  /// 该方法支持嵌套调用，内部使用 SAVEPOINT 机制：
  ///
  /// ```rust
  /// async fn nested_example(mm: &ModelManager) -> Result<()> {
  ///   mm.transaction(|mm| async move {
  ///     UserBmc::create(&mm, user1).await?;
  ///
  ///     // 嵌套事务
  ///     mm.transaction(|mm| async move {
  ///       UserBmc::create(&mm, user2).await?;
  ///       Ok(())
  ///     }).await?;
  ///
  ///     Ok(())
  ///   }).await?;
  ///   Ok(())
  /// }
  /// ```
  pub async fn transaction<F, Fut, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(ModelManager) -> Fut,
    Fut: Future<Output = Result<T>>,
  {
    let mm_txn = self.txn_cloned();
    mm_txn.dbx().begin_txn().await?;

    match f(mm_txn.clone()).await {
      Ok(result) => {
        mm_txn.dbx().commit_txn().await?;
        Ok(result)
      }
      Err(e) => {
        // 尝试回滚，如果失败则记录警告
        if let Err(rollback_err) = mm_txn.dbx().rollback_txn().await {
          log::warn!("Failed to rollback transaction: {:?}", rollback_err);
        }
        Err(e)
      }
    }
  }
}
