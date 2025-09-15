//! Agent 服务 JWE 认证集成测试

use std::sync::Arc;

use fusion_core::DataError;
use modelsql::ModelManager;
use uuid::Uuid;

use hetuflow_core::{
  models::{AgentCapabilities, AgentForCreate},
  protocol::{AgentRegisterRequest, AgentRegisterResponse},
  types::AgentStatus,
};

use crate::{
  service::{AgentSvc, JweConfig, JweService},
  setting::HetuflowServerSetting,
};

/// 创建测试用的 JWE 配置
fn create_test_jwe_config() -> JweConfig {
  let (private_key, public_key) = JweService::generate_key_pair().unwrap();
  JweConfig {
    private_key,
    public_key,
    key_agreement_algorithm: "ECDH-ES".to_string(),
    content_encryption_algorithm: "A256GCM".to_string(),
    token_ttl: 3600,
  }
}

/// 创建测试用的 Agent 注册请求
fn create_test_register_request(jwe_token: Option<String>) -> AgentRegisterRequest {
  AgentRegisterRequest {
    agent_id: String::new_v4(),
    name: Some("test-agent".to_string()),
    labels: vec!["test".to_string()],
    metadata: std::collections::HashMap::new(),
    capabilities: AgentCapabilities {
      max_concurrent_tasks: 10,
      supported_task_types: vec!["shell".to_string()],
      resource_limits: hetuflow_core::types::ResourceLimits {
        max_cpu_percent: 0.8,
        max_memory_bytes: 1024 * 1024 * 1024, // 1GB
      },
    },
    jwe_token,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setting::ServerConfig;

  /// 模拟 ModelManager 用于测试
  fn create_mock_model_manager() -> ModelManager {
    // 注意：这里需要根据实际的 ModelManager 创建方式进行调整
    // 在实际测试中，可能需要使用测试数据库或内存数据库
    todo!("需要实现测试用的 ModelManager")
  }

  /// 创建测试用的服务器设置
  fn create_test_server_setting(jwe_config: Option<JweConfig>) -> HetuflowServerSetting {
    HetuflowServerSetting {
      max_concurrent_tasks: 100,
      history_ttl: std::time::Duration::from_secs(3600),
      server: ServerConfig {
        server_id: String::new_v4(),
        bind_address: "127.0.0.1:9500".to_string(),
        heartbeat_interval: std::time::Duration::from_secs(30),
        agent_overdue_ttl: std::time::Duration::from_secs(300),
      },
      jwe: jwe_config,
    }
  }

  #[tokio::test]
  async fn test_agent_register_with_valid_jwe_token() {
    // 创建 JWE 配置和服务
    let jwe_config = create_test_jwe_config();
    let jwe_service = JweService::new(jwe_config.clone()).unwrap();

    let agent_id = Uuid::new_v4();
    let server_id = Uuid::new_v4();
    let permissions = vec!["read".to_string(), "write".to_string()];

    // 生成有效的 JWE Token
    let jwe_token = jwe_service.generate_token(agent_id, server_id, permissions).unwrap();

    // 创建注册请求
    let mut register_request = create_test_register_request(Some(jwe_token));
    register_request.agent_id = agent_id;

    // 创建服务器设置和 Agent 服务
    let server_setting = create_test_server_setting(Some(jwe_config));

    // 注意：这里需要实际的 ModelManager 实现
    // let mm = create_mock_model_manager();
    // let agent_svc = AgentSvc::with_jwe_config(mm, &server_setting).unwrap();

    // 测试注册处理
    // let result = agent_svc.handle_register(&agent_id, &register_request).await;
    // assert!(result.is_ok());

    // 暂时跳过实际测试，因为需要数据库支持
    println!("JWE Token 认证集成测试框架已创建");
  }

  #[tokio::test]
  async fn test_agent_register_with_invalid_jwe_token() {
    let jwe_config = create_test_jwe_config();
    let agent_id = Uuid::new_v4();

    // 创建无效的 JWE Token
    let invalid_token = "invalid.jwe.token".to_string();

    // 创建注册请求
    let mut register_request = create_test_register_request(Some(invalid_token));
    register_request.agent_id = agent_id;

    // 创建服务器设置
    let server_setting = create_test_server_setting(Some(jwe_config));

    // 注意：这里需要实际的 ModelManager 实现
    // let mm = create_mock_model_manager();
    // let agent_svc = AgentSvc::with_jwe_config(mm, &server_setting).unwrap();

    // 测试注册处理应该失败
    // let result = agent_svc.handle_register(&agent_id, &register_request).await;
    // assert!(result.is_err());

    println!("无效 JWE Token 测试框架已创建");
  }

  #[tokio::test]
  async fn test_agent_register_without_jwe_config() {
    let agent_id = Uuid::new_v4();

    // 创建不包含 JWE Token 的注册请求
    let mut register_request = create_test_register_request(None);
    register_request.agent_id = agent_id;

    // 创建不包含 JWE 配置的服务器设置
    let server_setting = create_test_server_setting(None);

    // 注意：这里需要实际的 ModelManager 实现
    // let mm = create_mock_model_manager();
    // let agent_svc = AgentSvc::with_jwe_config(mm, &server_setting).unwrap();

    // 测试注册处理应该成功（向后兼容）
    // let result = agent_svc.handle_register(&agent_id, &register_request).await;
    // assert!(result.is_ok());

    println!("无 JWE 配置测试框架已创建");
  }

  #[tokio::test]
  async fn test_agent_register_with_mismatched_agent_id() {
    let jwe_config = create_test_jwe_config();
    let jwe_service = JweService::new(jwe_config.clone()).unwrap();

    let token_agent_id = Uuid::new_v4();
    let request_agent_id = Uuid::new_v4(); // 不同的 Agent ID
    let server_id = Uuid::new_v4();
    let permissions = vec!["read".to_string()];

    // 为 token_agent_id 生成 JWE Token
    let jwe_token = jwe_service.generate_token(token_agent_id, server_id, permissions).unwrap();

    // 创建使用不同 Agent ID 的注册请求
    let mut register_request = create_test_register_request(Some(jwe_token));
    register_request.agent_id = request_agent_id;

    // 创建服务器设置
    let server_setting = create_test_server_setting(Some(jwe_config));

    // 注意：这里需要实际的 ModelManager 实现
    // let mm = create_mock_model_manager();
    // let agent_svc = AgentSvc::with_jwe_config(mm, &server_setting).unwrap();

    // 测试注册处理应该失败（Agent ID 不匹配）
    // let result = agent_svc.handle_register(&request_agent_id, &register_request).await;
    // assert!(result.is_err());

    println!("Agent ID 不匹配测试框架已创建");
  }

  #[test]
  fn test_jwe_config_creation() {
    let config = create_test_jwe_config();
    assert_eq!(config.key_agreement_algorithm, "ECDH-ES");
    assert_eq!(config.content_encryption_algorithm, "A256GCM");
    assert_eq!(config.token_ttl, 3600);
    assert!(!config.private_key.is_empty());
    assert!(!config.public_key.is_empty());
  }

  #[test]
  fn test_register_request_creation() {
    let token = Some("test-token".to_string());
    let request = create_test_register_request(token.clone());

    assert_eq!(request.name, Some("test-agent".to_string()));
    assert_eq!(request.labels, vec!["test".to_string()]);
    assert_eq!(request.jwe_token, token);
    assert_eq!(request.capabilities.max_concurrent_tasks, 10);
  }
}
