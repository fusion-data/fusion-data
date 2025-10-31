//! Integration Tests for Cluster Node Architecture
//!
//! This module provides comprehensive integration tests that verify the complete
//! functionality of the Cluster Node architecture with all SubNodeProviders.

use serde_json::json;
use std::sync::Arc;

use crate::workflow::{
  NodeExecutionError, NodeKind, NodeRegistry,
  graph_flow_tasks::ClusterNodeExecutor,
  providers::{
    AiAgentProvider, AiAgentProviderConfig, DeepSeekConfig, DeepSeekLLMProvider, MemoryProvider, MemoryProviderConfig,
  },
  sub_node_provider::{
    AgentConfig, AgentSubNodeProvider, ClusterNodeConfig, LLMConfig, MemoryConfig, Message, SubNodeProvider,
  },
};

/// Comprehensive integration test for all SubNodeProviders
pub async fn test_complete_cluster_node_integration() -> Result<(), NodeExecutionError> {
  println!("🚀 Starting complete Cluster Node integration test...");

  // 1. Create NodeRegistry
  let node_registry = NodeRegistry::new();

  // 2. Create and register DeepSeek LLM Provider
  println!("📝 Creating DeepSeek LLM Provider...");
  let deepseek_config = DeepSeekConfig {
    model: "deepseek-chat".to_string(),
    api_key: Some("test-api-key".to_string()),
    max_tokens: Some(2000),
    temperature: Some(0.7),
    ..Default::default()
  };
  let deepseek_provider = Arc::new(DeepSeekLLMProvider::new(deepseek_config.clone()));
  deepseek_provider.initialize().await?;

  let deepseek_kind: NodeKind = "deepseek_llm".into();
  let deepseek_provider_for_registry = deepseek_provider.clone();
  node_registry.register_subnode_provider(deepseek_kind.clone(), deepseek_provider_for_registry)?;
  println!("✅ DeepSeek LLM Provider registered successfully");

  // 3. Create and register Memory Provider
  println!("💾 Creating Memory Provider...");
  let memory_config = MemoryProviderConfig {
    max_messages: 100,
    persistence_enabled: true,
    session_timeout_seconds: 1800,
    cleanup_interval_seconds: 300,
  };
  let memory_provider = Arc::new(MemoryProvider::new(memory_config.clone()));
  memory_provider.initialize().await?;

  let memory_kind: NodeKind = "memory_provider".into();
  let memory_provider_for_registry = memory_provider.clone();
  node_registry.register_subnode_provider(memory_kind.clone(), memory_provider_for_registry)?;
  println!("✅ Memory Provider registered successfully");

  // 4. Create and register AI Agent Provider
  println!("🤖 Creating AI Agent Provider...");
  let agent_config = AiAgentProviderConfig {
    default_system_prompt: "You are a helpful AI assistant that can analyze data and provide insights.".to_string(),
    max_iterations: 5,
    default_temperature: 0.7,
    enable_tools: false, // Disable for this test
    enable_streaming: false,
    session_timeout_seconds: 3600,
  };
  let agent_provider = Arc::new(AiAgentProvider::new(agent_config.clone()));
  agent_provider.initialize().await?;

  let agent_kind: NodeKind = "ai_agent_provider".into();
  node_registry.register_subnode_provider(agent_kind.clone(), agent_provider)?;
  println!("✅ AI Agent Provider registered successfully");

  // 5. Verify all providers are registered
  assert_eq!(node_registry.subnode_provider_count(), 3);
  assert!(node_registry.has_subnode_provider(&deepseek_kind));
  assert!(node_registry.has_subnode_provider(&memory_kind));
  assert!(node_registry.has_subnode_provider(&agent_kind));
  println!("✅ All providers registered and verified");

  // 6. Create ClusterNodeExecutor and register tasks
  println!("⚡ Creating ClusterNodeExecutor...");
  let mut executor = ClusterNodeExecutor::new(node_registry.clone());

  // Register DeepSeek LLM task
  let deepseek_cluster_config = ClusterNodeConfig {
    llm_config: Some(LLMConfig {
      model: deepseek_config.model.clone(),
      max_tokens: deepseek_config.max_tokens,
      temperature: deepseek_config.temperature,
      top_p: None,
      stop_sequences: None,
      api_key: deepseek_config.api_key,
    }),
    ..Default::default()
  };
  executor.register_subnode_provider(deepseek_kind.clone(), deepseek_cluster_config)?;
  println!("✅ DeepSeek LLM task registered");

  // Register Memory task
  let memory_cluster_config = ClusterNodeConfig {
    memory_config: Some(MemoryConfig {
      context_window: Some(10),
      max_history: Some(50),
      persistence_enabled: Some(true),
    }),
    ..Default::default()
  };
  executor.register_subnode_provider(memory_kind.clone(), memory_cluster_config)?;
  println!("✅ Memory task registered");

  // Register AI Agent task
  let agent_cluster_config = ClusterNodeConfig {
    agent_config: Some(AgentConfig {
      system_prompt: Some("You are a data analysis assistant.".to_string()),
      max_iterations: Some(3),
      temperature: Some(0.6),
      enable_tools: Some(false),
      session_id: Some("integration_test_session".to_string()),
      ..Default::default()
    }),
    ..Default::default()
  };
  executor.register_subnode_provider(agent_kind.clone(), agent_cluster_config)?;
  println!("✅ AI Agent task registered");

  // 7. Verify all tasks are registered
  assert_eq!(executor.task_count(), 3);
  let task_ids = executor.task_ids();
  println!("✅ All tasks registered: {:?}", task_ids);

  // 8. Execute individual provider tests

  // Test Memory Provider
  println!("💾 Testing Memory Provider...");
  let memory_task_id = &task_ids.iter().find(|id| id.contains("memory_provider")).unwrap();
  let mut memory_context = executor.create_context();
  memory_context.set("test_session_id", "memory_test_session")?;

  let memory_result = executor.execute_task(memory_task_id, memory_context).await?;
  assert!(memory_result.response.is_some());
  println!("✅ Memory Provider test passed: {:?}", memory_result.response);

  // Test DeepSeek LLM Provider
  println!("📝 Testing DeepSeek LLM Provider...");
  let llm_task_id = &task_ids.iter().find(|id| id.contains("deepseek_llm")).unwrap();
  let mut llm_context = executor.create_context();
  llm_context.set("test_prompt", "Hello, this is a test message.")?;

  let llm_result = executor.execute_task(llm_task_id, llm_context).await?;
  assert!(llm_result.response.is_some());
  println!("✅ DeepSeek LLM Provider test passed: {:?}", llm_result.response);

  // Test AI Agent Provider
  println!("🤖 Testing AI Agent Provider...");
  let agent_task_id = &task_ids.iter().find(|id| id.contains("ai_agent_provider")).unwrap();
  let mut agent_context = executor.create_context();

  let test_messages = json!([
      {
          "role": "user",
          "content": "Please analyze the following numbers and tell me the trend: [1, 2, 4, 8, 16, 32]"
      }
  ]);
  agent_context.set("input_messages", test_messages)?;

  let agent_result = executor.execute_task(agent_task_id, agent_context).await?;
  assert!(agent_result.response.is_some());
  println!("✅ AI Agent Provider test passed: {:?}", agent_result.response);

  // 9. Test provider interoperability
  println!("🔗 Testing provider interoperability...");

  // Create an enhanced AI Agent with memory and LLM providers
  let enhanced_agent_provider = AiAgentProvider::new(agent_config)
    .with_llm_provider(deepseek_provider.clone())
    .with_memory_provider(memory_provider.clone());
  enhanced_agent_provider.initialize().await?;

  // Test multi-turn conversation
  let session_id = "multi_turn_test_session";
  let messages1 = vec![Message {
    role: "user".to_string(),
    content: "My name is Alice and I'm working on a data analysis project.".to_string(),
  }];

  let response1 = enhanced_agent_provider
    .execute_agent(
      messages1.clone(),
      AgentConfig { session_id: Some(session_id.to_string()), max_iterations: Some(2), ..Default::default() },
    )
    .await?;

  // Second message to test memory
  let messages2 = vec![Message {
    role: "user".to_string(),
    content: "What did I tell you about myself in the previous message?".to_string(),
  }];

  let response2 = enhanced_agent_provider
    .execute_agent(
      messages2.clone(),
      AgentConfig { session_id: Some(session_id.to_string()), max_iterations: Some(2), ..Default::default() },
    )
    .await?;

  println!("✅ Multi-turn conversation test passed");
  println!("   First response: {}", response1.content);
  println!("   Second response: {}", response2.content);

  // 10. Performance and statistics test
  println!("📊 Testing performance and statistics...");

  // Execute multiple concurrent requests
  let mut handles = Vec::new();

  for i in 0..5 {
    let agent_provider_clone = enhanced_agent_provider.clone();
    let handle = tokio::spawn(async move {
      let messages = vec![Message { role: "user".to_string(), content: format!("This is test message #{}", i + 1) }];

      agent_provider_clone
        .execute_agent(
          messages,
          AgentConfig {
            session_id: Some(format!("perf_test_session_{}", i)),
            max_iterations: Some(1),
            ..Default::default()
          },
        )
        .await
    });
    handles.push(handle);
  }

  // Wait for all requests to complete
  for (i, handle) in handles.into_iter().enumerate() {
    let result = handle.await.expect("Task should complete");
    match result {
      Ok(response) => {
        println!("✅ Concurrent request #{} completed", i + 1);
        assert!(!response.content.is_empty());
        assert!(response.usage.is_some());
      }
      Err(e) => {
        return Err(e);
      }
    }
  }

  println!("📈 Performance test completed successfully");

  // 11. Final verification
  println!("🔍 Final verification...");

  // Verify provider counts
  assert_eq!(node_registry.subnode_provider_count(), 3);
  assert_eq!(executor.task_count(), 3);

  // Verify provider definitions
  let deepseek_def = node_registry
    .get_subnode_provider(&deepseek_kind)
    .ok_or_else(|| NodeExecutionError::ConfigurationError("DeepSeek provider not found".to_string()))?
    .get_node_definition();
  assert_eq!(deepseek_def.kind.as_str(), "deepseek_llm");

  let memory_def = node_registry
    .get_subnode_provider(&memory_kind)
    .ok_or_else(|| NodeExecutionError::ConfigurationError("Memory provider not found".to_string()))?
    .get_node_definition();
  assert_eq!(memory_def.kind.as_str(), "memory_provider");

  let agent_def = node_registry
    .get_subnode_provider(&agent_kind)
    .ok_or_else(|| NodeExecutionError::ConfigurationError("Agent provider not found".to_string()))?
    .get_node_definition();
  assert_eq!(agent_def.kind.as_str(), "ai_agent_provider");

  println!("✅ All provider definitions verified");

  println!("🎉 Complete Cluster Node integration test passed successfully!");
  println!("📋 Test Summary:");
  println!("   - ✅ DeepSeek LLM Provider: Registered and functional");
  println!("   - ✅ Memory Provider: Registered and functional");
  println!("   - ✅ AI Agent Provider: Registered and functional");
  println!("   - ✅ Provider Interoperability: Working correctly");
  println!("   - ✅ Performance: Concurrent requests handled successfully");
  println!("   - ✅ Integration: All components working together");

  Ok(())
}

/// Test individual provider isolation and independence
pub async fn test_provider_isolation() -> Result<(), NodeExecutionError> {
  println!("🔒 Testing provider isolation...");

  // Create separate registries to test isolation
  let registry1 = NodeRegistry::new();
  let registry2 = NodeRegistry::new();

  // Register DeepSeek in registry1
  let deepseek_provider = Arc::new(DeepSeekLLMProvider::new(DeepSeekConfig::default()));
  deepseek_provider.initialize().await?;

  let deepseek_kind: NodeKind = "deepseek_llm_isolated".into();
  registry1.register_subnode_provider(deepseek_kind.clone(), deepseek_provider)?;

  // Register Memory in registry2
  let memory_provider = Arc::new(MemoryProvider::new(MemoryProviderConfig::default()));
  memory_provider.initialize().await?;

  let memory_kind: NodeKind = "memory_provider_isolated".into();
  registry2.register_subnode_provider(memory_kind.clone(), memory_provider)?;

  // Verify isolation
  assert_eq!(registry1.subnode_provider_count(), 1);
  assert_eq!(registry2.subnode_provider_count(), 1);
  assert!(registry1.has_subnode_provider(&deepseek_kind));
  assert!(!registry1.has_subnode_provider(&memory_kind));
  assert!(registry2.has_subnode_provider(&memory_kind));
  assert!(!registry2.has_subnode_provider(&deepseek_kind));

  println!("✅ Provider isolation test passed");

  Ok(())
}

/// Test error handling and recovery
pub async fn test_error_handling() -> Result<(), NodeExecutionError> {
  println!("⚠️ Testing error handling...");

  let registry = NodeRegistry::new();

  // Test invalid provider registration
  let invalid_provider = Arc::new(DeepSeekLLMProvider::new(DeepSeekConfig {
    api_key: None, // This should cause initialization to fail
    ..Default::default()
  }));

  let invalid_kind: NodeKind = "invalid_provider".into();

  // This should work for registration but fail during initialization
  let invalid_provider_for_registry = invalid_provider.clone();
  let result = registry.register_subnode_provider(invalid_kind.clone(), invalid_provider_for_registry);
  assert!(result.is_ok());

  // Test initialization failure
  let init_result = invalid_provider.initialize().await;
  assert!(init_result.is_err());

  // Test provider lookup
  let lookup_result = registry.get_subnode_provider(&invalid_kind);
  assert!(lookup_result.is_some());

  println!("✅ Error handling test passed");

  Ok(())
}

/// Run all integration tests
pub async fn run_all_integration_tests() -> Result<(), NodeExecutionError> {
  println!("🧪 Starting comprehensive integration test suite...\n");

  // Run individual tests
  test_provider_isolation().await?;
  println!();

  test_error_handling().await?;
  println!();

  test_complete_cluster_node_integration().await?;

  println!("\n🏅 All integration tests passed successfully!");
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_isolation_unit() {
    let result = test_provider_isolation().await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_error_handling_unit() {
    let result = test_error_handling().await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_complete_integration() {
    let result = test_complete_cluster_node_integration().await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_all_integration() {
    let result = run_all_integration_tests().await;
    assert!(result.is_ok());
  }
}
