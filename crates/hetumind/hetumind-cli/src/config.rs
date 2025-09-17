use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{CliError, CliResult};

/// API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
  /// API 服务端点地址
  pub endpoint: String,
  /// 认证令牌
  pub token: String,
}

impl Default for ApiConfig {
  fn default() -> Self {
    Self { endpoint: "http://127.0.0.1:8080".to_string(), token: String::new() }
  }
}

/// CLI 配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CliConfig {
  /// API 配置
  pub api: ApiConfig,
}

impl CliConfig {
  /// 从默认配置文件路径加载配置
  pub fn load() -> CliResult<Self> {
    let config_path = Self::default_config_path()?;
    Self::load_from_path(&config_path)
  }

  /// 从指定路径加载配置文件
  pub fn load_from_path(path: &PathBuf) -> CliResult<Self> {
    if !path.exists() {
      // 如果配置文件不存在，创建默认配置
      let config = Self::default();
      config.save_to_path(path)?;
      return Ok(config);
    }

    let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path.clone(), e))?;

    let config: CliConfig =
      toml::from_str(&content).map_err(|e| CliError::config_error(format!("配置文件解析失败: {}", e)))?;

    Ok(config)
  }

  /// 保存配置到指定路径
  pub fn save_to_path(&self, path: &PathBuf) -> CliResult<()> {
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).map_err(|e| CliError::io_error(parent.to_path_buf(), e))?;
    }

    let content = toml::to_string_pretty(self).map_err(|e| CliError::config_error(format!("配置序列化失败: {}", e)))?;

    std::fs::write(path, content).map_err(|e| CliError::io_error(path.clone(), e))?;

    Ok(())
  }

  /// 保存配置到默认路径
  pub fn save(&self) -> CliResult<()> {
    let config_path = Self::default_config_path()?;
    self.save_to_path(&config_path)
  }

  /// 获取默认配置文件路径：~/.hetumind/config.toml
  /// 支持通过环境变量 HETUMIND_CONFIG_PATH 覆盖默认路径
  pub fn default_config_path() -> CliResult<PathBuf> {
    // 优先使用环境变量指定的配置路径
    if let Ok(config_path) = std::env::var("HETUMIND_CONFIG_PATH") {
      return Ok(PathBuf::from(config_path));
    }

    let home_dir = dirs::home_dir().ok_or_else(|| CliError::config_error("无法获取用户主目录"))?;

    Ok(home_dir.join(".hetumind").join("config.toml"))
  }

  /// 验证配置的有效性
  pub fn validate(&self) -> CliResult<()> {
    if self.api.endpoint.is_empty() {
      return Err(CliError::validation_error("API endpoint 不能为空"));
    }

    if self.api.token.is_empty() {
      return Err(CliError::validation_error("API token 不能为空"));
    }

    // 验证 endpoint 格式
    if !self.api.endpoint.starts_with("http://") && !self.api.endpoint.starts_with("https://") {
      return Err(CliError::validation_error("API endpoint 必须以 http:// 或 https:// 开头"));
    }

    Ok(())
  }

  /// 检查配置是否有效（不抛出错误）
  pub fn is_valid(&self) -> bool {
    self.validate().is_ok()
  }

  /// 获取完整的 API URL
  pub fn api_url(&self, path: &str) -> String {
    let endpoint = self.api.endpoint.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{}/api/v1/{}", endpoint, path)
  }

  /// 获取认证头
  pub fn auth_header(&self) -> String {
    format!("Bearer {}", self.api.token)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use fusion_common::env::set_env;
  use tempfile::tempdir;

  #[test]
  fn test_config_serialization() {
    let config =
      CliConfig { api: ApiConfig { endpoint: "http://localhost:8080".to_string(), token: "test-token".to_string() } };

    let toml_str = toml::to_string_pretty(&config).unwrap();
    let parsed: CliConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(config.api.endpoint, parsed.api.endpoint);
    assert_eq!(config.api.token, parsed.api.token);
  }

  #[test]
  fn test_config_load_and_save() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let original_config = CliConfig {
      api: ApiConfig { endpoint: "http://test.example.com".to_string(), token: "test-secret".to_string() },
    };

    // 保存配置
    original_config.save_to_path(&config_path).unwrap();

    // 加载配置
    let loaded_config = CliConfig::load_from_path(&config_path).unwrap();

    assert_eq!(original_config.api.endpoint, loaded_config.api.endpoint);
    assert_eq!(original_config.api.token, loaded_config.api.token);
  }

  #[test]
  fn test_config_validation() {
    let mut config = CliConfig::default();

    // 空 token 应该验证失败
    assert!(config.validate().is_err());
    assert!(!config.is_valid());

    // 设置 token 但 endpoint 格式错误
    config.api.token = "test-token".to_string();
    config.api.endpoint = "invalid-url".to_string();
    assert!(config.validate().is_err());
    assert!(!config.is_valid());

    // 正确的配置应该验证成功
    config.api.endpoint = "http://localhost:8080".to_string();
    assert!(config.validate().is_ok());
    assert!(config.is_valid());
  }

  #[test]
  fn test_api_url_generation() {
    let config =
      CliConfig { api: ApiConfig { endpoint: "http://localhost:8080".to_string(), token: "test-token".to_string() } };

    assert_eq!(config.api_url("workflows"), "http://localhost:8080/api/v1/workflows");
    assert_eq!(config.api_url("/workflows"), "http://localhost:8080/api/v1/workflows");
    assert_eq!(config.api_url("workflows/123"), "http://localhost:8080/api/v1/workflows/123");
  }

  #[test]
  fn test_config_path_from_env() {
    let temp_dir = tempdir().unwrap();
    let custom_config = temp_dir.path().join("custom_config.toml");

    // 设置环境变量
    set_env("HETUMIND_CONFIG_PATH", custom_config.to_str().unwrap()).unwrap();

    let config_path = CliConfig::default_config_path().unwrap();
    assert_eq!(config_path, custom_config);

    // 清理环境变量
    unsafe {
      std::env::remove_var("HETUMIND_CONFIG_PATH");
    }
  }
}
