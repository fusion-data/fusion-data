use std::sync::Arc;

use fusion_common::ctx::Ctx;
use fusion_core::{DataError, application::Application};
use fusionsql::{ModelManager, SqlError};

use crate::{
  model::{CtxExt, TenantAccessValidator, TenantFilter, policy::TenantAccessMode},
  web::JieyuanClient,
};

/// 平台管理员服务
/// 提供平台管理员特有的功能和权限管理
#[derive(Clone)]
pub struct PlatformAdminService {
  app: Arc<Application>,
  model_manager: Arc<ModelManager>,
  jieyuan_client: Arc<JieyuanClient>,
}

impl PlatformAdminService {
  /// 创建新的平台管理员服务
  pub fn new(app: Arc<Application>, model_manager: Arc<ModelManager>, jieyuan_client: Arc<JieyuanClient>) -> Self {
    Self { app, model_manager, jieyuan_client }
  }

  /// 检查用户是否为平台管理员
  pub async fn is_platform_admin(&self, ctx: &Ctx) -> bool {
    ctx.is_platform_admin()
  }

  /// 获取用户可访问的租户列表
  pub async fn get_accessible_tenants(&self, ctx: &Ctx) -> Result<Vec<i64>, DataError> {
    if !self.is_platform_admin(ctx).await {
      return Err(DataError::Unauthorized("Only platform administrators can access tenant lists".to_string()));
    }

    let filter = TenantFilter::new(ctx.clone());
    match ctx.tenant_access_mode() {
      TenantAccessMode::All => {
        // 返回所有租户
        self.get_all_tenants().await
      }
      TenantAccessMode::Current => Ok(vec![ctx.tenant_id()]),
      TenantAccessMode::Specific => {
        let managed_ids = ctx.managed_tenant_ids();
        let tenant_ids: Vec<i64> = managed_ids.iter().filter_map(|id| id.parse().ok()).collect();
        self.validate_and_filter_tenants(&tenant_ids).await
      }
    }
  }

  /// 设置用户的租户访问模式
  pub async fn set_tenant_access_mode(
    &self,
    ctx: &Ctx,
    target_user_id: i64,
    access_mode: TenantAccessMode,
    managed_tenants: Option<Vec<i64>>,
  ) -> Result<(), DataError> {
    if !self.is_platform_admin(ctx).await {
      return Err(DataError::Unauthorized("Only platform administrators can set tenant access modes".to_string()));
    }

    // 验证目标用户存在
    if !self.user_exists(target_user_id).await? {
      return Err(DataError::NotFound(format!("User {} not found", target_user_id)));
    }

    // 验证管理租户列表（如果指定）
    if let (TenantAccessMode::Specific, Some(tenant_ids)) = (&access_mode, &managed_tenants) {
      self.validate_tenant_list(tenant_ids).await?;
    }

    // 保存配置到数据库
    self.save_user_tenant_config(target_user_id, access_mode, managed_tenants).await
  }

  /// 跨租户查询数据
  pub async fn cross_tenant_query<T, F, R>(
    &self,
    ctx: &Ctx,
    tenant_ids: &[i64],
    query_fn: F,
  ) -> Result<Vec<R>, DataError>
  where
    F: Fn(i64) -> T,
    T: std::future::Future<Output = Result<R, DataError>>,
  {
    if !self.is_platform_admin(ctx).await {
      return Err(DataError::Unauthorized("Only platform administrators can perform cross-tenant queries".to_string()));
    }

    let validator = TenantAccessValidator::new(ctx.clone());
    validator.validate_tenant_list_access(tenant_ids)?;

    let mut results = Vec::new();
    for &tenant_id in tenant_ids {
      let result = query_fn(tenant_id).await?;
      results.push(result);
    }

    Ok(results)
  }

  /// 创建租户切换令牌
  pub async fn create_tenant_switch_token(
    &self,
    ctx: &Ctx,
    target_tenant_id: i64,
    reason: Option<String>,
  ) -> Result<String, DataError> {
    if !self.is_platform_admin(ctx).await {
      return Err(DataError::Unauthorized("Only platform administrators can create tenant switch tokens".to_string()));
    }

    let validator = TenantAccessValidator::new(ctx.clone());
    validator.validate_tenant_access(target_tenant_id)?;

    // 创建临时访问令牌
    let token = self.generate_temporary_token(ctx, target_tenant_id, reason).await?;
    Ok(token)
  }

  /// 获取平台管理员的审计日志
  pub async fn get_admin_audit_logs(
    &self,
    ctx: &Ctx,
    filters: AdminAuditLogFilters,
  ) -> Result<Vec<AdminAuditLog>, DataError> {
    if !self.is_platform_admin(ctx).await {
      return Err(DataError::Unauthorized("Only platform administrators can access audit logs".to_string()));
    }

    self.query_audit_logs(filters).await
  }

  // --- 私有辅助方法 ---

  /// 获取所有租户
  async fn get_all_tenants(&self) -> Result<Vec<i64>, DataError> {
    // 实现获取所有租户的逻辑
    // 这里简化处理，实际应该查询租户表
    Ok(vec![])
  }

  /// 验证租户列表
  async fn validate_tenant_list(&self, tenant_ids: &[i64]) -> Result<(), DataError> {
    for &tenant_id in tenant_ids {
      if !self.tenant_exists(tenant_id).await? {
        return Err(DataError::NotFound(format!("Tenant {} not found", tenant_id)));
      }
    }
    Ok(())
  }

  /// 验证并过滤租户列表
  async fn validate_and_filter_tenants(&self, tenant_ids: &[i64]) -> Result<Vec<i64>, DataError> {
    let mut valid_tenants = Vec::new();
    for &tenant_id in tenant_ids {
      if self.tenant_exists(tenant_id).await? {
        valid_tenants.push(tenant_id);
      }
    }
    Ok(valid_tenants)
  }

  /// 检查用户是否存在
  async fn user_exists(&self, user_id: i64) -> Result<bool, DataError> {
    // 实现用户存在性检查
    // 这里简化处理
    Ok(true)
  }

  /// 检查租户是否存在
  async fn tenant_exists(&self, tenant_id: i64) -> Result<bool, DataError> {
    // 实现租户存在性检查
    // 这里简化处理
    Ok(true)
  }

  /// 保存用户租户配置
  async fn save_user_tenant_config(
    &self,
    user_id: i64,
    access_mode: TenantAccessMode,
    managed_tenants: Option<Vec<i64>>,
  ) -> Result<(), DataError> {
    // 实现保存用户租户配置的逻辑
    // 这里简化处理，实际应该保存到数据库
    Ok(())
  }

  /// 生成临时令牌
  async fn generate_temporary_token(
    &self,
    ctx: &Ctx,
    target_tenant_id: i64,
    reason: Option<String>,
  ) -> Result<String, DataError> {
    // 实现临时令牌生成逻辑
    // 这里简化处理，实际应该生成JWT令牌
    Ok(format!("temp_token_{}_{}", ctx.user_id(), target_tenant_id))
  }

  /// 查询审计日志
  async fn query_audit_logs(&self, filters: AdminAuditLogFilters) -> Result<Vec<AdminAuditLog>, DataError> {
    // 实现审计日志查询逻辑
    // 这里简化处理
    Ok(vec![])
  }
}

/// 管理员审计日志过滤器
#[derive(Debug, Clone, Default)]
pub struct AdminAuditLogFilters {
  pub start_time: Option<chrono::DateTime<chrono::FixedOffset>>,
  pub end_time: Option<chrono::DateTime<chrono::FixedOffset>>,
  pub user_id: Option<i64>,
  pub tenant_id: Option<i64>,
  pub action: Option<String>,
  pub limit: Option<u32>,
  pub offset: Option<u32>,
}

/// 管理员审计日志
#[derive(Debug, Clone)]
pub struct AdminAuditLog {
  pub id: i64,
  pub user_id: i64,
  pub tenant_id: Option<i64>,
  pub action: String,
  pub resource: String,
  pub details: Option<serde_json::Value>,
  pub ip_address: String,
  pub user_agent: Option<String>,
  pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

/// 平台管理员权限管理器
#[derive(Clone)]
pub struct PlatformAdminPermissionManager {
  platform_admin_service: Arc<PlatformAdminService>,
}

impl PlatformAdminPermissionManager {
  /// 创建新的权限管理器
  pub fn new(platform_admin_service: Arc<PlatformAdminService>) -> Self {
    Self { platform_admin_service }
  }

  /// 检查用户是否有特定权限
  pub async fn has_permission(&self, ctx: &Ctx, permission: &str, resource: Option<&str>) -> Result<bool, DataError> {
    if !ctx.is_platform_admin() {
      return Ok(false);
    }

    // 基于权限类型和资源进行验证
    match permission {
      "tenant.read" => Ok(true),
      "tenant.write" => self.check_tenant_write_permission(ctx, resource).await,
      "tenant.switch" => self.check_tenant_switch_permission(ctx, resource).await,
      "user.read" => Ok(true),
      "user.write" => self.check_user_write_permission(ctx, resource).await,
      "system.admin" => Ok(true),
      _ => Ok(false),
    }
  }

  /// 获取用户的有效权限列表
  pub async fn get_effective_permissions(&self, ctx: &Ctx) -> Result<Vec<String>, DataError> {
    if !ctx.is_platform_admin(ctx).await {
      return Ok(vec![]);
    }

    let mut permissions = vec!["system.admin".to_string(), "user.read".to_string()];

    // 根据租户访问模式添加权限
    match ctx.tenant_access_mode() {
      TenantAccessMode::All => {
        permissions.extend_from_slice(&[
          "tenant.read".to_string(),
          "tenant.write".to_string(),
          "tenant.switch".to_string(),
          "user.write".to_string(),
        ]);
      }
      TenantAccessMode::Specific => {
        permissions.extend_from_slice(&["tenant.read".to_string(), "tenant.switch".to_string()]);

        // 检查是否有写权限
        if self.has_specific_tenant_write_permission(ctx).await? {
          permissions.push("tenant.write".to_string());
        }
      }
      TenantAccessMode::Current => {
        permissions.push("tenant.read".to_string());
      }
    }

    Ok(permissions)
  }

  // --- 私有辅助方法 ---

  async fn check_tenant_write_permission(&self, ctx: &Ctx, resource: Option<&str>) -> Result<bool, DataError> {
    match ctx.tenant_access_mode() {
      TenantAccessMode::All => Ok(true),
      TenantAccessMode::Specific => self.has_specific_tenant_write_permission(ctx).await,
      TenantAccessMode::Current => Ok(resource.is_none() || resource == Some(&ctx.tenant_id().to_string())),
    }
  }

  async fn check_tenant_switch_permission(&self, ctx: &Ctx, resource: Option<&str>) -> Result<bool, DataError> {
    match ctx.tenant_access_mode() {
      TenantAccessMode::All => Ok(true),
      TenantAccessMode::Specific => {
        if let Some(tenant_id_str) = resource {
          if let Ok(tenant_id) = tenant_id_str.parse::<i64>() {
            let validator = TenantAccessValidator::new(ctx.clone());
            Ok(validator.can_access_tenant(tenant_id))
          } else {
            Ok(false)
          }
        } else {
          Ok(true)
        }
      }
      TenantAccessMode::Current => Ok(false),
    }
  }

  async fn check_user_write_permission(&self, ctx: &Ctx, resource: Option<&str>) -> Result<bool, DataError> {
    match ctx.tenant_access_mode() {
      TenantAccessMode::All => Ok(true),
      TenantAccessMode::Specific => self.has_specific_tenant_write_permission(ctx).await,
      TenantAccessMode::Current => Ok(false),
    }
  }

  async fn has_specific_tenant_write_permission(&self, ctx: &Ctx) -> Result<bool, DataError> {
    // 检查用户是否有特定租户的写权限
    // 这里简化处理，实际应该检查具体权限配置
    Ok(true)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use fusion_common::ctx::{Ctx, CtxPayload};

  fn create_platform_admin_ctx() -> Ctx {
    let mut payload = CtxPayload::default();
    payload.set_i64("sub", 1);
    payload.set_i64("tenant_id", 100);
    payload.set_bool("is_platform_admin", true);
    payload.set_string("tenant_access_mode", "all");

    Ctx::try_new(payload, None, None).unwrap()
  }

  #[tokio::test]
  async fn test_platform_admin_permissions() {
    let ctx = create_platform_admin_ctx();

    // 创建模拟服务
    let app = Arc::new(Application::new_default());
    let model_manager = Arc::new(ModelManager::new());
    let jieyuan_client = Arc::new(JieyuanClient::new("http://localhost:8080".to_string()));

    let service = PlatformAdminService::new(app, model_manager, jieyuan_client);
    let permission_manager = PlatformAdminPermissionManager::new(Arc::new(service));

    // 测试权限检查
    assert!(permission_manager.has_permission(&ctx, "system.admin", None).await.unwrap());
    assert!(permission_manager.has_permission(&ctx, "tenant.read", None).await.unwrap());
    assert!(permission_manager.has_permission(&ctx, "tenant.write", None).await.unwrap());

    // 测试有效权限列表
    let permissions = permission_manager.get_effective_permissions(&ctx).await.unwrap();
    assert!(permissions.contains(&"system.admin".to_string()));
    assert!(permissions.contains(&"tenant.read".to_string()));
  }
}
