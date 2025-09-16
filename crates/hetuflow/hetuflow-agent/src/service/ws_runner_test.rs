//! WebSocket Runner JWE 认证测试

use std::sync::Arc;
use uuid::Uuid;

use hetuflow_core::{models::AgentCapabilities, protocol::AgentRegisterRequest, types::ResourceLimits};

use crate::setting::{ConnectionConfig, HetuflowAgentSetting, PollingConfig, ProcessConfig};

/// 创建测试用的 Agent 设置
fn create_test_agent_setting(jwe_token: Option<String>) -> HetuflowAgentSetting {
  HetuflowAgentSetting {
    agent_id: String::new_v4(),
    name: Some("test-agent".to_string()),
    labels: vec!["test".to_string()],
    work_dir: Some("/tmp/test".to_string()),
    metadata: std::collections::HashMap::new(),
    jwe_token,
    connection: Arc::new(ConnectionConfig {
      connect_timeout: std::time::Duration::from_secs(30),
      heartbeat_interval: std::time::Duration::from_secs(30),
      max_reconnect_attempts: 10,
      reconnect_interval: std::time::Duration::from_secs(10),
      server_address: "127.0.0.1:9500".to_string(),
    }),
    polling: Arc::new(PollingConfig {
      max_concurrent_tasks: 20,
      enable_adaptive_polling: true,
      interval_seconds: 60,
      capacity_weight: 50,
      load_factor_threshold: 1.0,
    }),
    process: Arc::new(ProcessConfig {
      enable_resource_monitoring: true,
      resource_monitor_interval: std::time::Duration::from_secs(30),
      cleanup_interval: std::time::Duration::from_secs(3600),
      max_concurrent_processes: 8,
      process_timeout: std::time::Duration::from_secs(7200),
      zombie_check_interval: std::time::Duration::from_secs(3600),
      limits: ResourceLimits { max_cpu_percent: 0.7, max_memory_bytes: 4194304 },
    }),
  }
}

/// 创建测试用的 Agent 能力
fn create_test_agent_capabilities() -> AgentCapabilities {
  AgentCapabilities {
    max_concurrent_tasks: 20,
    supported_task_types: vec!["shell".to_string(), "python".to_string()],
    resource_limits: ResourceLimits { max_cpu_percent: 0.7, max_memory_bytes: 4194304 },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_agent_register_request_with_jwe_token() {
    let jwe_token = Some("test-jwe-token-12345".to_string());
    let setting = create_test_agent_setting(jwe_token.clone());
    let capabilities = create_test_agent_capabilities();

    // 模拟创建 AgentRegisterRequest 的过程
    let register_request = AgentRegisterRequest {
      agent_id: setting.agent_id,
      name: setting.name.clone(),
      labels: setting.labels.clone(),
      metadata: setting.metadata.clone(),
      capabilities,
      jwe_token: setting.jwe_token.clone(),
    };

    // 验证 JWE Token 被正确包含
    assert_eq!(register_request.jwe_token, jwe_token);
    assert_eq!(register_request.agent_id, setting.agent_id);
    assert_eq!(register_request.name, setting.name);
    assert_eq!(register_request.labels, setting.labels);
  }

  #[test]
  fn test_agent_register_request_without_jwe_token() {
    let setting = create_test_agent_setting(None);
    let capabilities = create_test_agent_capabilities();

    // 模拟创建 AgentRegisterRequest 的过程
    let register_request = AgentRegisterRequest {
      agent_id: setting.agent_id,
      name: setting.name.clone(),
      labels: setting.labels.clone(),
      metadata: setting.metadata.clone(),
      capabilities,
      jwe_token: setting.jwe_token.clone(),
    };

    // 验证没有 JWE Token
    assert_eq!(register_request.jwe_token, None);
    assert_eq!(register_request.agent_id, setting.agent_id);
  }

  #[test]
  fn test_agent_setting_jwe_token_configuration() {
    // 测试有 JWE Token 的配置
    let token = "eyJhbGciOiJFQ0RILUVTK0EyNTZHQ00iLCJlbmMiOiJBMjU2R0NNIn0...".to_string();
    let setting_with_token = create_test_agent_setting(Some(token.clone()));
    assert_eq!(setting_with_token.jwe_token, Some(token));

    // 测试没有 JWE Token 的配置
    let setting_without_token = create_test_agent_setting(None);
    assert_eq!(setting_without_token.jwe_token, None);
  }

  #[test]
  fn test_agent_capabilities_creation() {
    let capabilities = create_test_agent_capabilities();

    assert_eq!(capabilities.max_concurrent_tasks, 20);
    assert_eq!(capabilities.supported_task_types, vec!["shell", "python"]);
    assert_eq!(capabilities.resource_limits.max_cpu_percent, 0.7);
    assert_eq!(capabilities.resource_limits.max_memory_bytes, 4194304);
  }

  #[test]
  fn test_server_gateway_ws_url_generation() {
    let setting = create_test_agent_setting(None);
    let expected_url = format!("ws://{}/ws", setting.connection.server_address);
    let actual_url = setting.server_gateway_ws();

    assert_eq!(actual_url, expected_url);
  }

  #[test]
  fn test_connection_config_values() {
    let setting = create_test_agent_setting(None);

    assert_eq!(setting.connection.connect_timeout, std::time::Duration::from_secs(30));
    assert_eq!(setting.connection.heartbeat_interval, std::time::Duration::from_secs(30));
    assert_eq!(setting.connection.max_reconnect_attempts, 10);
    assert_eq!(setting.connection.reconnect_interval, std::time::Duration::from_secs(10));
    assert_eq!(setting.connection.server_address, "127.0.0.1:9500");
  }

  #[test]
  fn test_polling_config_values() {
    let setting = create_test_agent_setting(None);

    assert_eq!(setting.polling.max_concurrent_tasks, 20);
    assert_eq!(setting.polling.enable_adaptive_polling, true);
    assert_eq!(setting.polling.interval_seconds, 60);
    assert_eq!(setting.polling.capacity_weight, 50);
    assert_eq!(setting.polling.load_factor_threshold, 1.0);
  }

  #[test]
  fn test_process_config_values() {
    let setting = create_test_agent_setting(None);

    assert_eq!(setting.process.enable_resource_monitoring, true);
    assert_eq!(setting.process.resource_monitor_interval, std::time::Duration::from_secs(30));
    assert_eq!(setting.process.cleanup_interval, std::time::Duration::from_secs(3600));
    assert_eq!(setting.process.max_concurrent_processes, 8);
    assert_eq!(setting.process.process_timeout, std::time::Duration::from_secs(7200));
    assert_eq!(setting.process.zombie_check_interval, std::time::Duration::from_secs(3600));
    assert_eq!(setting.process.limits.max_cpu_percent, 0.7);
    assert_eq!(setting.process.limits.max_memory_bytes, 4194304);
  }

  /// 测试 JWE Token 在不同场景下的行为
  #[test]
  fn test_jwe_token_scenarios() {
    // 场景1：空字符串 Token
    let empty_token_setting = create_test_agent_setting(Some("".to_string()));
    assert_eq!(empty_token_setting.jwe_token, Some("".to_string()));

    // 场景2：长 Token
    let long_token = "a".repeat(1000);
    let long_token_setting = create_test_agent_setting(Some(long_token.clone()));
    assert_eq!(long_token_setting.jwe_token, Some(long_token));

    // 场景3：包含特殊字符的 Token
    let special_token = "token.with-special_chars+/=".to_string();
    let special_token_setting = create_test_agent_setting(Some(special_token.clone()));
    assert_eq!(special_token_setting.jwe_token, Some(special_token));
  }
}
