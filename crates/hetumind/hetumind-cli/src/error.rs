use std::path::PathBuf;
use thiserror::Error;

/// CLI 统一错误类型
#[derive(Error, Debug)]
pub enum CliError {
  /// 文件读写错误
  #[error("文件操作失败: {path}")]
  IoError {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  /// 配置加载或解析错误
  #[error("配置错误: {message}")]
  ConfigError { message: String },

  /// API 请求失败 (包括网络错误和服务器返回的错误)
  #[error("API 请求失败: {message}")]
  ApiError { message: String },

  /// 数据验证失败
  #[error("数据验证失败: {message}")]
  ValidationError { message: String },

  /// JSON 序列化/反序列化错误
  #[error("JSON 处理错误: {0}")]
  JsonError(#[from] serde_json::Error),

  /// 配置文件格式错误
  #[error("配置文件格式错误: {0}")]
  TomlError(#[from] toml::de::Error),

  /// HTTP 客户端错误
  #[error("HTTP 请求错误: {0}")]
  HttpError(#[from] reqwest::Error),
}

/// CLI 结果类型
pub type CliResult<T> = Result<T, CliError>;

impl CliError {
  /// 创建配置错误
  pub fn config_error(message: impl Into<String>) -> Self {
    Self::ConfigError { message: message.into() }
  }

  /// 创建 API 错误
  pub fn api_error(message: impl Into<String>) -> Self {
    Self::ApiError { message: message.into() }
  }

  /// 创建验证错误
  pub fn validation_error(message: impl Into<String>) -> Self {
    Self::ValidationError { message: message.into() }
  }

  /// 创建文件操作错误
  pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
    Self::IoError { path: path.into(), source }
  }
}
