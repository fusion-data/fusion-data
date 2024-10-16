use fusion_server::ctx::CtxW;
use modql::filter::OpValString;
use ultimate::Result;

use super::{GlobalPath, GlobalPathBmc, GlobalPathFilter};

pub struct GlobalPathSvc;
impl GlobalPathSvc {
  pub async fn find_unique(ctx: &CtxW, path: impl Into<String>) -> Result<Option<GlobalPath>> {
    GlobalPathBmc::find_unique(
      ctx.mm(),
      vec![GlobalPathFilter { path: Some(OpValString::Eq(path.into()).into()), ..Default::default() }],
    )
    .await
    .map_err(Into::into)
  }

  pub async fn find_many(ctx: &CtxW, filter: GlobalPathFilter) -> Result<Vec<GlobalPath>> {
    GlobalPathBmc::find_many(ctx.mm(), vec![filter], None).await.map_err(Into::into)
  }

  pub async fn obtain_lock(ctx: &CtxW, path: &str, value: Option<String>) -> Result<bool> {
    GlobalPathBmc::obtain_lock(ctx.mm(), path, value, None).await.map_err(Into::into)
  }

  pub async fn obtain_optimistic_lock(ctx: &CtxW, path: &str, revision: i64, value: Option<String>) -> Result<bool> {
    GlobalPathBmc::obtain_lock(ctx.mm(), path, value, Some(revision)).await.map_err(Into::into)
  }
}
