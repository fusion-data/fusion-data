use fusion_server::ctx::CtxW;
use ultimate::Result;

use super::AssociateNamespaceWithScheduler;

pub struct SchedNamespaceSvc;
impl SchedNamespaceSvc {
  pub async fn associate_scheduler(ctx: &CtxW, datas: Vec<AssociateNamespaceWithScheduler>) -> Result<()> {
    todo!()
  }
}
