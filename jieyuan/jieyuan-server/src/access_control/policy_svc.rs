use axum::extract::FromRequestParts;
use fusion_common::ctx::Ctx;
use fusion_core::{Result, application::Application};
use fusion_web::WebError;
use fusionsql::{ModelManager, page::PageResult};

use jieyuan_core::model::{
  CtxExt, Decision, DecisionEffect, PolicyEngine, PolicyEntity, PolicyForCreate, PolicyForPage, PolicyForUpdate,
};

use super::{PolicyRepo, policy_bmc::PolicyBmc};

/// 权限服务（Arc 并发友好）
#[derive(Clone)]
pub struct PolicySvc {
  pub(crate) repo: PolicyRepo,
  // 保留原有的 ModelManager 用于策略管理操作
  mm: ModelManager,
}

impl PolicySvc {
  /// 函数级注释：创建带数据库仓库的权限服务
  pub fn new(mm: ModelManager) -> Self {
    let repo = PolicyRepo::new(mm.clone());
    Self { repo, mm }
  }

  /// 函数级注释：执行授权判断
  pub async fn authorize_ext(&self, ctx: &Ctx, action: &str, resource: &str) -> Result<Decision> {
    // 1) 聚合策略集
    let mut policies = Vec::new();
    policies.extend(self.repo.list_attached_policies_for_user(ctx.tenant_id(), ctx.user_id()).await?);
    policies.extend(self.repo.list_policies_for_roles(ctx.tenant_id(), &ctx.roles()).await?);
    policies.extend(self.repo.list_resource_policies(ctx.tenant_id(), resource).await?);

    let boundary = self.repo.find_permission_boundary(ctx.tenant_id(), ctx.user_id()).await?;
    let session = self.repo.find_session_policy("current").await?;

    // 2) 求值：显式拒绝优先
    if PolicyEngine::match_any(&policies, ctx, action, resource, DecisionEffect::Deny) {
      return Ok(Decision::Deny);
    }

    let allowed = PolicyEngine::match_any(&policies, ctx, action, resource, DecisionEffect::Allow);
    if !allowed {
      return Ok(Decision::Deny);
    }

    // 3) 边界与会话策略裁剪
    if let Some(pb) = boundary
      && !PolicyEngine::match_policy(&pb, ctx, action, resource, DecisionEffect::Allow)
    {
      return Ok(Decision::Deny);
    }
    if let Some(sp) = session
      && !PolicyEngine::match_policy(&sp, ctx, action, resource, DecisionEffect::Allow)
    {
      return Ok(Decision::Deny);
    }

    Ok(Decision::Allow)
  }

  // 以下为策略管理的 CRUD 操作（保留原有功能）

  pub async fn create(&self, policy_for_create: PolicyForCreate) -> Result<i64> {
    let id = PolicyBmc::create(&self.mm, policy_for_create).await?;
    Ok(id)
  }

  pub async fn find_option_by_id(&self, id: i64) -> Result<Option<PolicyEntity>> {
    let policy = PolicyBmc::find_by_id(&self.mm, id).await.ok();
    Ok(policy)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<PolicyEntity> {
    let policy = PolicyBmc::find_by_id(&self.mm, id).await?;
    Ok(policy)
  }

  pub async fn update_by_id(&self, id: i64, policy_for_update: PolicyForUpdate) -> Result<()> {
    PolicyBmc::update_by_id(&self.mm, id, policy_for_update).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, id: i64) -> Result<()> {
    PolicyBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  pub async fn page(&self, req: PolicyForPage) -> Result<PageResult<PolicyEntity>> {
    let page = PolicyBmc::page(&self.mm, req.filter, req.page).await?;
    Ok(page)
  }
}

// 为 PolicySvc 实现 FromRequestParts trait，使其可以从请求中提取
impl FromRequestParts<Application> for PolicySvc {
  type Rejection = WebError;

  async fn from_request_parts(
    _parts: &mut axum::http::request::Parts,
    state: &Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    let mm = state.component::<ModelManager>();
    Ok(PolicySvc::new(mm))
  }
}
