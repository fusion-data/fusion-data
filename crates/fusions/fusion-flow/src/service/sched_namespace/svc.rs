use fusiondata_context::ctx::CtxW;
use ultimate::{component::Component, Result};
use ultimate_api::v1::Pagination;

use super::{SchedNamespace, SchedNamespaceBmc, SchedNamespaceFilter, SchedNamespaceForUpdate};

#[derive(Clone, Component)]
pub struct SchedNamespaceSvc {}

impl SchedNamespaceSvc {
  pub async fn find_many(
    &self,
    ctx: &CtxW,
    filter: Vec<SchedNamespaceFilter>,
    pagination: Option<&Pagination>,
  ) -> Result<Vec<SchedNamespace>> {
    SchedNamespaceBmc::find_many(ctx.mm(), filter, pagination).await.map_err(Into::into)
  }

  pub async fn update(
    &self,
    ctx: &CtxW,
    filter: Vec<SchedNamespaceFilter>,
    entity_u: SchedNamespaceForUpdate,
  ) -> Result<u64> {
    SchedNamespaceBmc::update(ctx.mm(), filter, entity_u).await.map_err(Into::into)
  }

  pub async fn count(&self, ctx: &CtxW, filter: Vec<SchedNamespaceFilter>) -> Result<i64> {
    SchedNamespaceBmc::count(ctx.mm(), filter).await.map_err(Into::into)
  }
}
