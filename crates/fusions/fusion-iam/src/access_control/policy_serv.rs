use fusiondata::ac::abac::policy::PolicyStatement;
use fusiondata_context::ctx::CtxW;
use ultimate::Result;
use uuid::Uuid;

use crate::access_control::{bmc::PolicyBmc, PolicyForCreate};

use super::Policy;

pub async fn create(ctx: &CtxW, policy: PolicyStatement, description: Option<String>) -> Result<Uuid> {
  let id = Uuid::now_v7();
  let entity_c = PolicyForCreate { id, description, policy: serde_json::to_value(policy)?, status: Some(100) };
  PolicyBmc::insert(ctx.mm(), entity_c).await?;

  Ok(id)
}

pub async fn find_by_id(ctx: &CtxW, id: Uuid) -> Result<Policy> {
  let policy = PolicyBmc::find_by_id(ctx.mm(), id).await?;
  Ok(policy)
}
