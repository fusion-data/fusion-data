use fusion_common::ctx::Ctx;
use fusionsql::{Fields, filter::OpValInt64};

use crate::model::CtxExt;

/// 租户过滤器 - 用于数据查询层面的动态租户过滤
/// 支持平台管理员跨租户查询和普通用户的租户隔离
#[derive(Debug, Clone)]
pub struct TenantFilter {
  ctx: Ctx,
}

impl TenantFilter {
  /// 创建新的租户过滤器
  pub fn new(ctx: Ctx) -> Self {
    Self { ctx }
  }

  /// 获取上下文
  pub fn ctx(&self) -> &Ctx {
    &self.ctx
  }

  /// 检查是否可以访问指定租户的数据
  pub fn can_access_tenant(&self, tenant_id: i64) -> bool {
    self.ctx.can_access_tenant(tenant_id)
  }

  /// 获取可访问的租户ID列表
  pub fn accessible_tenant_ids(&self) -> Vec<i64> {
    match self.ctx.tenant_access_mode() {
      crate::model::policy::TenantAccessMode::All => {
        // 平台管理员可访问所有租户，返回None表示不过滤
        vec![]
      }
      crate::model::policy::TenantAccessMode::Current => {
        vec![self.ctx.tenant_id()]
      }
      crate::model::policy::TenantAccessMode::Specific => {
        self.ctx.managed_tenant_ids().iter().filter_map(|id| id.parse().ok()).collect()
      }
    }
  }

  /// 检查是否需要应用租户过滤
  pub fn needs_tenant_filter(&self) -> bool {
    !matches!(self.ctx.tenant_access_mode(), crate::model::policy::TenantAccessMode::All)
      || !self.ctx.is_platform_admin()
  }

  /// 生成租户过滤条件（用于sea-query）
  /// 返回 (需要过滤, 租户ID列表, 是否全租户访问)
  pub fn tenant_filter_condition(&self) -> (bool, Vec<i64>, bool) {
    let tenant_ids = self.accessible_tenant_ids();
    let all_access = self.ctx.is_platform_admin()
      && matches!(self.ctx.tenant_access_mode(), crate::model::policy::TenantAccessMode::All);

    (self.needs_tenant_filter(), tenant_ids, all_access)
  }
}

/// 可租户过滤的查询特征
/// 实现此特征的查询结构可以自动应用租户过滤
pub trait TenantFilteredQuery: Fields {
  /// 应用于查询的租户过滤条件
  fn apply_tenant_filter(&mut self, filter: &TenantFilter);

  /// 获取租户ID字段名（默认为 "tenant_id"）
  fn tenant_id_field() -> &'static str {
    "tenant_id"
  }
}

/// 租户过滤查询构建器
#[derive(Debug, Clone)]
pub struct TenantFilteredQueryBuilder<Q> {
  query: Q,
  tenant_filter: Option<TenantFilter>,
}

impl<Q> TenantFilteredQueryBuilder<Q> {
  /// 创建新的查询构建器
  pub fn new(query: Q) -> Self {
    Self { query, tenant_filter: None }
  }

  /// 设置租户过滤器
  pub fn with_tenant_filter(mut self, ctx: Ctx) -> Self {
    self.tenant_filter = Some(TenantFilter::new(ctx));
    self
  }

  /// 构建最终查询
  pub fn build(mut self) -> Q
  where
    Q: TenantFilteredQuery,
  {
    if let Some(filter) = &self.tenant_filter {
      self.query.apply_tenant_filter(filter);
    }
    self.query
  }

  /// 获取内部查询引用
  pub fn query(&self) -> &Q {
    &self.query
  }

  /// 获取内部查询可变引用
  pub fn query_mut(&mut self) -> &mut Q {
    &mut self.query
  }
}

/// 辅助函数：为查询应用租户过滤
pub fn apply_tenant_filter_to_query<Q>(mut query: Q, ctx: &Ctx, field_name: Option<&str>) -> Q
where
  Q: TenantFilteredQuery,
{
  let filter = TenantFilter::new(ctx.clone());
  if let Some(field_name) = field_name {
    // 这里可以扩展支持自定义字段名
    // 暂时使用默认字段名
  }
  query.apply_tenant_filter(&filter);
  query
}

/// 租户访问权限验证器
#[derive(Debug, Clone)]
pub struct TenantAccessValidator {
  ctx: Ctx,
}

impl TenantAccessValidator {
  /// 创建新的访问权限验证器
  pub fn new(ctx: Ctx) -> Self {
    Self { ctx }
  }

  /// 验证租户访问权限
  pub fn validate_tenant_access(&self, tenant_id: i64) -> Result<(), TenantAccessError> {
    if !self.ctx.can_access_tenant(tenant_id) {
      return Err(TenantAccessError::AccessDenied {
        user_id: self.ctx.user_id(),
        tenant_id,
        reason: format!("User with access mode {:?} cannot access tenant {}", self.ctx.tenant_access_mode(), tenant_id),
      });
    }
    Ok(())
  }

  /// 验证租户列表访问权限
  pub fn validate_tenant_list_access(&self, tenant_ids: &[i64]) -> Result<(), TenantAccessError> {
    for &tenant_id in tenant_ids {
      self.validate_tenant_access(tenant_id)?;
    }
    Ok(())
  }

  /// 过滤可访问的租户ID列表
  pub fn filter_accessible_tenants(&self, tenant_ids: &[i64]) -> Vec<i64> {
    tenant_ids.iter().copied().filter(|&id| self.ctx.can_access_tenant(id)).collect()
  }
}

/// 租户访问错误类型
#[derive(Debug, thiserror::Error)]
pub enum TenantAccessError {
  #[error("Access denied for user {user_id} to tenant {tenant_id}: {reason}")]
  AccessDenied { user_id: i64, tenant_id: i64, reason: String },

  #[error("Invalid tenant access configuration: {0}")]
  InvalidConfiguration(String),
}

impl From<TenantAccessError> for fusion_core::DataError {
  fn from(err: TenantAccessError) -> Self {
    fusion_core::DataError::BadRequest(err.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use fusion_common::ctx::{Ctx, CtxPayload};
  use std::collections::HashMap;

  fn create_test_ctx(
    user_id: i64,
    tenant_id: i64,
    is_platform_admin: bool,
    tenant_access_mode: &str,
    managed_tenants: Vec<&str>,
  ) -> Ctx {
    let mut payload = CtxPayload::default();
    payload.set_i64("sub", user_id);
    payload.set_i64("tenant_id", tenant_id);
    payload.set_bool("is_platform_admin", is_platform_admin);
    payload.set_string("tenant_access_mode", tenant_access_mode);
    payload.set_strings("managed_tenant_ids", managed_tenants);

    Ctx::try_new(payload, None, None).unwrap()
  }

  #[test]
  fn test_tenant_filter_normal_user() {
    let ctx = create_test_ctx(1, 100, false, "current", vec![]);
    let filter = TenantFilter::new(ctx);

    assert!(filter.can_access_tenant(100));
    assert!(!filter.can_access_tenant(200));
    assert_eq!(filter.accessible_tenant_ids(), vec![100]);
    assert!(filter.needs_tenant_filter());
  }

  #[test]
  fn test_tenant_filter_platform_admin_all() {
    let ctx = create_test_ctx(1, 100, true, "all", vec![]);
    let filter = TenantFilter::new(ctx);

    assert!(filter.can_access_tenant(100));
    assert!(filter.can_access_tenant(200));
    assert!(filter.accessible_tenant_ids().is_empty());
    assert!(!filter.needs_tenant_filter());
  }

  #[test]
  fn test_tenant_filter_platform_admin_specific() {
    let ctx = create_test_ctx(1, 100, true, "specific", vec!["100", "200", "300"]);
    let filter = TenantFilter::new(ctx);

    assert!(filter.can_access_tenant(100));
    assert!(filter.can_access_tenant(200));
    assert!(filter.can_access_tenant(300));
    assert!(!filter.can_access_tenant(400));
    assert_eq!(filter.accessible_tenant_ids(), vec![100, 200, 300]);
    assert!(filter.needs_tenant_filter());
  }

  #[test]
  fn test_tenant_access_validator() {
    let ctx = create_test_ctx(1, 100, true, "specific", vec!["100", "200"]);
    let validator = TenantAccessValidator::new(ctx);

    assert!(validator.validate_tenant_access(100).is_ok());
    assert!(validator.validate_tenant_access(200).is_ok());
    assert!(validator.validate_tenant_access(300).is_err());

    let filtered = validator.filter_accessible_tenants(&[100, 200, 300, 400]);
    assert_eq!(filtered, vec![100, 200]);
  }
}
