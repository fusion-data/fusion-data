use fusion_common::ctx::Ctx;

/// Context 扩展 trait，为 fusion-common::Ctx 提供 IAM 相关的便捷方法
/// 直接使用 Ctx 和 ctx.payload() 方法，避免与内置方法冲突
pub trait CtxExt {
  /// 检查用户是否拥有指定角色
  fn has_role(&self, role: &str) -> bool;

  /// 检查用户是否平台管理员
  fn is_platform_admin(&self) -> bool;

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
    self.payload().get_strings("principal_roles").unwrap_or_default().iter().any(|r| *r == role)
  }

  fn is_platform_admin(&self) -> bool {
    self.payload().get_bool("is_platform_admin").unwrap_or(false)
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
