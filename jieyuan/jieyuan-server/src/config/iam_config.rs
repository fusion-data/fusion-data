use serde::Deserialize;

/// IAM 配置结构
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IamConfig {
  /// 是否启用资源策略（默认 true）
  #[serde(default = "default_enable_resource_policies")]
  pub enable_resource_policies: bool,
  /// 是否启用权限边界（默认 false）
  #[serde(default = "default_enable_permission_boundary")]
  pub enable_permission_boundary: bool,
}

impl Default for IamConfig {
  fn default() -> Self {
    Self { enable_resource_policies: true, enable_permission_boundary: false }
  }
}

fn default_enable_resource_policies() -> bool {
  true
}

fn default_enable_permission_boundary() -> bool {
  false
}
