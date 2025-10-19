use fusion_common::ctx::Ctx;

use crate::model::policy::TenantAccessMode;

/// Context 扩展 trait，为 fusion-common::Ctx 提供 IAM 相关的便捷方法
/// 直接使用 Ctx 和 ctx.payload() 方法，避免与内置方法冲突
///
/// 注意：直接从用户登录的会话 tokon(jwe) 中只能获取到此 trait 定义的 token_seq, request_timestamp 数据，
/// 其它数据需要服务通过调用 /api/v1/authorize/authorize 返回的 ctx 获取
pub trait CtxExt {
  /// 检查用户是否拥有指定角色
  fn has_role(&self, role: &str) -> bool;

  /// 检查用户是否平台管理员
  fn is_platform_admin(&self) -> bool;

  /// 获取用户的租户访问模式
  fn tenant_access_mode(&self) -> TenantAccessMode;

  /// 获取用户可管理的租户ID列表（当模式为Specific时）
  fn managed_tenant_ids(&self) -> Vec<String>;

  /// 检查用户是否可以访问指定租户
  fn can_access_tenant(&self, tenant_id: i64) -> bool;

  /// 获取令牌序列号
  fn token_seq(&self) -> i32;

  /// 获取请求方法
  fn request_method(&self) -> &str;

  /// 获取请求路径
  fn request_path(&self) -> &str;

  /// 获取客户端 IP
  fn client_ip(&self) -> &str;

  /// 获取请求时间戳（秒）
  fn request_timestamp(&self) -> i64;

  /// 获取所有角色
  fn roles(&self) -> Vec<&str>;
}

impl CtxExt for Ctx {
  fn has_role(&self, role: &str) -> bool {
    self.payload().get_strings("principal_roles").unwrap_or_default().contains(&role)
  }

  fn is_platform_admin(&self) -> bool {
    self.payload().get_bool("is_platform_admin").unwrap_or(false)
  }

  fn tenant_access_mode(&self) -> TenantAccessMode {
    if !self.is_platform_admin() {
      return TenantAccessMode::Current;
    }

    match self.payload().get_str("tenant_access_mode") {
      Some("all") => TenantAccessMode::All,
      Some("specific") => TenantAccessMode::Specific,
      _ => TenantAccessMode::Current,
    }
  }

  fn managed_tenant_ids(&self) -> Vec<String> {
    self
      .payload()
      .get_strings("managed_tenant_ids")
      .unwrap_or_default()
      .into_iter()
      .map(|s| s.to_string())
      .collect()
  }

  fn can_access_tenant(&self, tenant_id: i64) -> bool {
    match self.tenant_access_mode() {
      TenantAccessMode::Current => self.tenant_id() == tenant_id,
      TenantAccessMode::All => true,
      TenantAccessMode::Specific => {
        let managed_ids = self.managed_tenant_ids();
        managed_ids.contains(&tenant_id.to_string())
      }
    }
  }

  fn token_seq(&self) -> i32 {
    self.payload().get_i32("token_seq").unwrap_or(0)
  }

  fn request_method(&self) -> &str {
    self.payload().get_str("method").unwrap_or_default()
  }

  fn request_path(&self) -> &str {
    self.payload().get_str("path").unwrap_or_default()
  }

  fn client_ip(&self) -> &str {
    self.payload().get_str("request_ip").unwrap_or_default()
  }

  fn request_timestamp(&self) -> i64 {
    self.req_epoch_secs()
  }

  fn roles(&self) -> Vec<&str> {
    self.payload().get_strings("principal_roles").unwrap_or_default()
  }
}
