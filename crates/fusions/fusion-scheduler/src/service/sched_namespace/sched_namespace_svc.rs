use fusion_server::ctx::CtxW;
use ultimate::Result;

use super::{AssociateNamespaceWithScheduler, SchedNamespace, SchedNamespaceBmc};

pub struct SchedNamespaceSvc;
impl SchedNamespaceSvc {
  pub async fn associate_scheduler(
    ctx: &CtxW,
    datas: Vec<AssociateNamespaceWithScheduler>,
  ) -> Result<Vec<SchedNamespace>> {
    todo!()
  }

  pub async fn find_many(ctx: &CtxW, filters: Vec<super::SchedNamespaceFilter>) -> Result<Vec<SchedNamespace>> {
    SchedNamespaceBmc::find_many(ctx.mm(), filters, None).await.map_err(Into::into)
  }
}
