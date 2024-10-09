use fusion_server::ctx::CtxW;
use ultimate::Result;

use super::{sched_node_bmc::SchedNodeBmc, SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate};

pub struct SchedNodeSvc;
impl SchedNodeSvc {
  pub async fn find_by_id(ctx: &CtxW, id: &str) -> Result<SchedNode> {
    let entity = SchedNodeBmc::find_by_id(ctx.mm(), id).await?;
    Ok(entity)
  }

  pub async fn create(ctx: &CtxW, entity_c: SchedNodeForCreate) -> Result<()> {
    SchedNodeBmc::insert(ctx.mm(), entity_c).await?;
    Ok(())
  }

  pub async fn update_by_id(ctx: &CtxW, id: &str, entity_u: SchedNodeForUpdate) -> Result<()> {
    SchedNodeBmc::update_by_id(ctx.mm(), id, entity_u).await?;
    Ok(())
  }

  pub async fn find(ctx: &CtxW, filter: Vec<SchedNodeFilter>) -> Result<Option<SchedNode>> {
    let opt = SchedNodeBmc::find_unique(ctx.mm(), filter).await?;
    Ok(opt)
  }
}
