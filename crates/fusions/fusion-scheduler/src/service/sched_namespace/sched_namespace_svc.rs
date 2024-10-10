use fusion_server::ctx::CtxW;
use ultimate::Result;

use super::{AssociateNamespaceWithScheduler, SchedNamespace};

pub struct SchedNamespaceSvc;
impl SchedNamespaceSvc {
  pub async fn associate_scheduler(
    ctx: &CtxW,
    datas: Vec<AssociateNamespaceWithScheduler>,
  ) -> Result<Vec<SchedNamespace>> {
    todo!()
  }

  /// 获取未使用（未关联或节点已离线）的 namespaces
  ///
  /// ```postgresql
  /// select * from sched.sched_namespace sn
  /// left join sched.sched_node s on s.id = sn.node_id
  /// where sn.node_id is null or s.status != 100 or s.last_check_time < now() - interval '30 seconds'
  /// ```
  pub async fn get_unused_namespace(ctx: &CtxW) -> Result<Vec<SchedNamespace>> {
    todo!()
  }
}
