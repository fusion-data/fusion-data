//! JWE Token 认证服务
//!
//! 提供基于 JWE (JSON Web Encryption) 标准的 Token 生成和验证功能
//! 采用 ECDH-ES 密钥协商 + A256GCM 内容加密算法

use chrono::Utc;
use fusion_core::DataError;
use josekit::{
  jwe::{JweContext, JweHeader},
  jwk::{
    Jwk,
    alg::ec::{EcCurve, EcKeyPair},
  },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// JWE 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JweConfig {
  /// 私钥 (PEM 格式)
  pub private_key: String,
  /// 公钥 (PEM 格式)
  pub public_key: String,
  /// 密钥协商算法 (默认: ECDH-ES)
  #[serde(default = "default_key_agreement_algorithm")]
  pub key_agreement_algorithm: String,
  /// 内容加密算法 (默认: A256GCM)
  #[serde(default = "default_content_encryption_algorithm")]
  pub content_encryption_algorithm: String,
  /// Token 有效期 (秒，默认: 永久)
  #[serde(default = "default_token_ttl")]
  pub token_ttl: u64,
}

fn default_key_agreement_algorithm() -> String {
  "ECDH-ES".to_string()
}

fn default_content_encryption_algorithm() -> String {
  "A256GCM".to_string()
}

fn default_token_ttl() -> u64 {
  0 // 永久有效
}

impl Default for JweConfig {
  fn default() -> Self {
    Self {
      private_key: String::new(),
      public_key: String::new(),
      key_agreement_algorithm: default_key_agreement_algorithm(),
      content_encryption_algorithm: default_content_encryption_algorithm(),
      token_ttl: default_token_ttl(),
    }
  }
}

/// JWE Token Payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JweTokenPayload {
  /// 标准 Claims
  pub iss: String, // Issuer
  pub sub: String, // Subject (agent_id)
  pub aud: String, // Audience
  pub exp: i64,    // Expiration Time
  pub nbf: i64,    // Not Before
  pub iat: i64,    // Issued At
  pub jti: String, // JWT ID

  /// 自定义 Claims
  pub server_id: String, // Server ID
  pub permissions: Vec<String>, // 权限列表
}

/// JWE 服务错误类型
#[derive(Debug, Error)]
pub enum JweServiceError {
  #[error("密钥格式错误: {0}")]
  InvalidKeyFormat(String),

  #[error("Token 生成失败: {0}")]
  TokenGenerationFailed(String),

  #[error("Token 解密失败: {0}")]
  TokenDecryptionFailed(String),

  #[error("Token 验证失败: {0}")]
  TokenValidationFailed(String),

  #[error("Agent ID 不匹配: expected {expected}, got {actual}")]
  AgentIdMismatch { expected: String, actual: String },

  #[error("Token 已过期")]
  TokenExpired,

  #[error("Token 尚未生效")]
  TokenNotYetValid,

  #[error("JSON 序列化/反序列化错误: {0}")]
  JsonError(#[from] serde_json::Error),
}
impl From<JweServiceError> for DataError {
  fn from(error: JweServiceError) -> Self {
    match error {
      JweServiceError::InvalidKeyFormat(msg) => DataError::bad_request(msg),
      JweServiceError::TokenGenerationFailed(msg) => DataError::server_error(msg),
      JweServiceError::TokenDecryptionFailed(msg) => DataError::bad_request(msg),
      JweServiceError::TokenValidationFailed(msg) => DataError::bad_request(msg),
      JweServiceError::AgentIdMismatch { expected, actual } => {
        DataError::bad_request(format!("Agent ID mismatch: expected {}, got {}", expected, actual))
      }
      JweServiceError::TokenExpired => DataError::unauthorized("Token has expired"),
      JweServiceError::TokenNotYetValid => DataError::unauthorized("Token is not yet valid"),
      JweServiceError::JsonError(e) => DataError::bad_request(e.to_string()),
    }
  }
}

/// JWE Token 认证服务
#[derive(Debug, Clone)]
pub struct JweService {
  config: JweConfig,
  private_key: Jwk,
  public_key: Jwk,
}

impl JweService {
  /// 创建新的 JWE 服务实例
  pub fn new(config: JweConfig) -> Result<Self, JweServiceError> {
    // 解析私钥 (PEM 格式)
    let ec_key_pair = EcKeyPair::from_pem(&config.private_key, Some(EcCurve::P256))
      .map_err(|e| JweServiceError::InvalidKeyFormat(format!("私钥解析失败: {}", e)))?;
    let private_key = ec_key_pair.to_jwk_private_key();

    // 解析公钥 (PEM 格式) - 从私钥生成对应的公钥
    let public_key = ec_key_pair.to_jwk_public_key();

    Ok(Self { config, private_key, public_key })
  }

  /// 生成 JWE Token
  pub fn generate_token(
    &self,
    agent_id: &str,
    server_id: &str,
    permissions: Vec<String>,
  ) -> Result<String, JweServiceError> {
    let now = Utc::now();
    let exp = if self.config.token_ttl > 0 { now.timestamp() + self.config.token_ttl as i64 } else { i64::MAX };
    let jti = Uuid::new_v4().to_string();

    // 创建 Payload
    let payload = JweTokenPayload {
      iss: "hetuflow-server".to_string(),
      sub: agent_id.to_string(),
      aud: "hetuflow-agent".to_string(),
      exp,
      nbf: now.timestamp(),
      iat: now.timestamp(),
      jti,
      server_id: server_id.to_string(),
      permissions,
    };

    // 序列化 Payload
    let payload_json = serde_json::to_string(&payload)?;

    // 创建 JWE Header
    let mut header = JweHeader::new();
    header.set_algorithm(&self.config.key_agreement_algorithm);
    header.set_content_encryption(&self.config.content_encryption_algorithm);
    header.set_token_type("JWE");

    // 使用 josekit 的 JWE 加密
    let context = JweContext::new();
    let encrypter = josekit::jwe::ECDH_ES
      .encrypter_from_jwk(&self.public_key)
      .map_err(|e| JweServiceError::TokenGenerationFailed(format!("创建加密器失败: {}", e)))?;
    let token = context
      .serialize_compact(payload_json.as_bytes(), &header, &encrypter)
      .map_err(|e| JweServiceError::TokenGenerationFailed(format!("JWE 加密失败: {}", e)))?;

    Ok(token)
  }

  /// 验证并解密 JWE Token
  pub fn verify_token(&self, token: &str, expected_agent_id: String) -> Result<JweTokenPayload, JweServiceError> {
    // 解密 JWE Token
    let context = JweContext::new();
    let decrypter = josekit::jwe::ECDH_ES
      .decrypter_from_jwk(&self.private_key)
      .map_err(|e| JweServiceError::TokenDecryptionFailed(format!("创建解密器失败: {}", e)))?;
    let (payload_bytes, _header) = context
      .deserialize_compact(token, &decrypter)
      .map_err(|e| JweServiceError::TokenDecryptionFailed(format!("JWE 解密失败: {}", e)))?;

    // 反序列化 Payload
    let payload: JweTokenPayload = serde_json::from_slice(&payload_bytes)?;

    // 验证时间
    let now = Utc::now().timestamp();
    if payload.exp < now {
      return Err(JweServiceError::TokenExpired);
    }
    if payload.nbf > now {
      return Err(JweServiceError::TokenNotYetValid);
    }

    // 验证 sub 字段与 agent_id 一致性
    if payload.sub != expected_agent_id {
      return Err(JweServiceError::TokenValidationFailed("sub 字段与 agent_id 不一致".to_string()));
    }

    Ok(payload)
  }

  /// 生成 ECDH-ES 密钥对
  pub fn generate_key_pair() -> Result<(String, String), JweServiceError> {
    let key_pair = EcKeyPair::generate(EcCurve::P256)
      .map_err(|e| JweServiceError::InvalidKeyFormat(format!("密钥生成失败: {}", e)))?;

    let private_key_pem = String::from_utf8(key_pair.to_pem_private_key())
      .map_err(|e| JweServiceError::InvalidKeyFormat(format!("私钥转换失败: {}", e)))?;

    let public_key_pem = String::from_utf8(key_pair.to_pem_public_key())
      .map_err(|e| JweServiceError::InvalidKeyFormat(format!("公钥转换失败: {}", e)))?;

    Ok((private_key_pem, public_key_pem))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_jwe_token_generation_and_verification() {
    // 生成测试密钥对
    let (private_key, public_key) = JweService::generate_key_pair().unwrap();

    let config = JweConfig {
      private_key,
      public_key,
      key_agreement_algorithm: "ECDH-ES".to_string(),
      content_encryption_algorithm: "A256GCM".to_string(),
      token_ttl: 3600,
    };

    let service = JweService::new(config).unwrap();
    let agent_id = Uuid::new_v4().to_string();
    let server_id = Uuid::new_v4().to_string();
    let permissions = vec!["read".to_string(), "write".to_string()];

    // 生成 Token
    let token = service.generate_token(&agent_id, &server_id, permissions.clone()).unwrap();
    assert!(!token.is_empty());

    // 验证 Token
    let payload = service.verify_token(&token, agent_id.clone()).unwrap();
    assert_eq!(payload.sub, agent_id);
    assert_eq!(payload.server_id, server_id);
    assert_eq!(payload.permissions, permissions);
  }

  #[tokio::test]
  async fn test_agent_id_mismatch() {
    let (private_key, public_key) = JweService::generate_key_pair().unwrap();

    let config = JweConfig {
      private_key,
      public_key,
      key_agreement_algorithm: "ECDH-ES".to_string(),
      content_encryption_algorithm: "A256GCM".to_string(),
      token_ttl: 3600,
    };

    let service = JweService::new(config).unwrap();
    let agent_id = Uuid::new_v4().to_string();
    let different_agent_id = Uuid::new_v4().to_string();
    let server_id = Uuid::new_v4().to_string();
    let permissions = vec!["read".to_string()];

    // 生成 Token
    let token = service.generate_token(&agent_id, &server_id, permissions).unwrap();

    // 使用不同的 agent_id 验证应该失败
    let result = service.verify_token(&token, different_agent_id);
    assert!(matches!(result, Err(JweServiceError::AgentIdMismatch { .. })));
  }

  #[tokio::test]
  async fn test_token_expiration() {
    let (private_key, public_key) = JweService::generate_key_pair().unwrap();

    let config = JweConfig {
      private_key,
      public_key,
      key_agreement_algorithm: "ECDH-ES".to_string(),
      content_encryption_algorithm: "A256GCM".to_string(),
      token_ttl: 1, // 1秒过期
    };

    let service = JweService::new(config).unwrap();
    let agent_id = Uuid::new_v4().to_string();
    let server_id = Uuid::new_v4().to_string();
    let permissions = vec!["read".to_string()];

    // 生成 Token
    let token = service.generate_token(&agent_id, &server_id, permissions).unwrap();

    // 等待 Token 过期
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 验证过期的 Token 应该失败
    let result = service.verify_token(&token, agent_id);
    assert!(matches!(result, Err(JweServiceError::TokenExpired)));
  }

  #[tokio::test]
  async fn test_invalid_token_format() {
    let (private_key, public_key) = JweService::generate_key_pair().unwrap();

    let config = JweConfig {
      private_key,
      public_key,
      key_agreement_algorithm: "ECDH-ES".to_string(),
      content_encryption_algorithm: "A256GCM".to_string(),
      token_ttl: 3600,
    };

    let service = JweService::new(config).unwrap();
    let agent_id = Uuid::new_v4().to_string();

    // 测试无效的 Token 格式
    let invalid_token = "invalid.token.format";
    let result = service.verify_token(invalid_token, agent_id.clone());
    assert!(matches!(result, Err(JweServiceError::TokenDecryptionFailed(_))));
  }

  #[tokio::test]
  async fn test_key_pair_generation() {
    let result = JweService::generate_key_pair();
    assert!(result.is_ok());

    let (private_key, public_key) = result.unwrap();
    assert!(!private_key.is_empty());
    assert!(!public_key.is_empty());
    assert!(private_key.contains("BEGIN PRIVATE KEY"));
    assert!(public_key.contains("BEGIN PUBLIC KEY"));
  }

  #[tokio::test]
  async fn test_jwe_config_default() {
    let config = JweConfig::default();
    assert_eq!(config.key_agreement_algorithm, "ECDH-ES");
    assert_eq!(config.content_encryption_algorithm, "A256GCM");
    assert_eq!(config.token_ttl, 3600);
  }

  #[tokio::test]
  async fn test_multiple_permissions() {
    let (private_key, public_key) = JweService::generate_key_pair().unwrap();

    let config = JweConfig {
      private_key,
      public_key,
      key_agreement_algorithm: "ECDH-ES".to_string(),
      content_encryption_algorithm: "A256GCM".to_string(),
      token_ttl: 3600,
    };

    let service = JweService::new(config).unwrap();
    let agent_id = Uuid::new_v4().to_string();
    let server_id = Uuid::new_v4().to_string();
    let permissions = vec!["read".to_string(), "write".to_string(), "execute".to_string(), "admin".to_string()];

    // 生成 Token
    let token = service.generate_token(&agent_id, &server_id, permissions.clone()).unwrap();

    // 验证 Token
    let payload = service.verify_token(&token, agent_id).unwrap();
    assert_eq!(payload.permissions, permissions);
    assert_eq!(payload.permissions.len(), 4);
  }
}
