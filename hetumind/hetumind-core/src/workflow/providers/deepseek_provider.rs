//! DeepSeek LLM SubNodeProvider Implementation
//!
//! This module provides a SubNodeProvider implementation for DeepSeek LLM models,
//! integrating with the Cluster Node architecture while maintaining compatibility
//! with existing DeepSeek functionality.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid;

use crate::workflow::{
  NodeDefinition, NodeExecutionError,
  sub_node_provider::{
    LLMConfig, LLMResponse, LLMSubNodeProvider, Message, SubNodeProvider, SubNodeProviderType, UsageStats,
  },
  workflow_node::NodeGroupKind,
};

use crate::version::Version;

/// DeepSeek-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
  /// API key for DeepSeek API
  pub api_key: Option<String>,
  /// Model name (e.g., "deepseek-chat", "deepseek-coder")
  pub model: String,
  /// Maximum tokens to generate
  pub max_tokens: Option<u32>,
  /// Temperature for response randomness (0.0-2.0)
  pub temperature: Option<f64>,
  /// Top-p sampling parameter
  pub top_p: Option<f64>,
  /// Stop sequences for generation
  pub stop_sequences: Option<Vec<String>>,
  /// Base URL for API (optional, uses default if not specified)
  pub base_url: Option<String>,
  /// Timeout in seconds
  pub timeout: Option<u64>,
}

impl Default for DeepSeekConfig {
  fn default() -> Self {
    Self {
      api_key: None,
      model: "deepseek-chat".to_string(),
      max_tokens: Some(4096),
      temperature: Some(0.7),
      top_p: Some(1.0),
      stop_sequences: None,
      base_url: None,
      timeout: Some(60),
    }
  }
}

impl From<LLMConfig> for DeepSeekConfig {
  fn from(llm_config: LLMConfig) -> Self {
    Self {
      api_key: llm_config.api_key,
      model: llm_config.model,
      max_tokens: llm_config.max_tokens,
      temperature: llm_config.temperature,
      top_p: llm_config.top_p.map(|v| v as f64),
      stop_sequences: llm_config.stop_sequences,
      base_url: None,
      timeout: None,
    }
  }
}

/// DeepSeek LLM SubNodeProvider implementation
#[derive(Debug)]
pub struct DeepSeekLLMProvider {
  config: DeepSeekConfig,
  node_definition: Arc<NodeDefinition>,
  provider_id: String,
}

impl DeepSeekLLMProvider {
  /// Create a new DeepSeek LLM provider
  pub fn new(config: DeepSeekConfig) -> Self {
    let provider_id = format!("deepseek_{}", uuid::Uuid::now_v7());

    // Create a simple node definition for DeepSeek provider
    // Note: This is a simplified version that avoids complex type conversions
    let node_definition = Arc::new(NodeDefinition {
      kind: "deepseek_llm".into(),
      version: Version::parse("1.0.0").unwrap(),
      groups: vec![NodeGroupKind::Transform],
      display_name: format!("DeepSeek LLM Provider ({})", config.model),
      description: Some("DeepSeek LLM provider for text generation and chat".to_string()),
      inputs: vec![],
      outputs: vec![],
      properties: vec![],
      document_url: Some("https://platform.deepseek.com/".to_string()),
      sub_title: Some("AI text generation".to_string()),
      hidden: false,
      max_nodes: None,
      icon: Some("robot".to_string()),
      icon_color: Some(crate::types::IconColor::Blue),
      icon_url: None,
      badge_icon_url: None,
    });

    Self { config, node_definition, provider_id }
  }

  /// Create provider from LLMConfig
  pub fn from_llm_config(llm_config: LLMConfig) -> Self {
    Self::new(DeepSeekConfig::from(llm_config))
  }

  /// Get the provider's unique ID
  pub fn provider_id(&self) -> &str {
    &self.provider_id
  }

  /// Get current configuration
  pub fn config(&self) -> &DeepSeekConfig {
    &self.config
  }

  /// Update configuration
  pub fn update_config(&mut self, config: DeepSeekConfig) {
    self.config = config;
  }

  /// Resolve API key from configuration or environment
  fn resolve_api_key(&self) -> Result<String, NodeExecutionError> {
    // Try to get API key from configuration first
    if let Some(api_key) = &self.config.api_key {
      return Ok(api_key.clone());
    }

    // Try to get from environment variable
    if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
      return Ok(api_key);
    }

    Err(NodeExecutionError::ConfigurationError("DeepSeek API key not found. Please set DEEPSEEK_API_KEY environment variable or provide api_key in configuration".to_string()))
  }

  /// Convert messages to DeepSeek API format
  fn convert_messages_to_deepseek_format(
    &self,
    messages: Vec<Message>,
  ) -> Result<Vec<serde_json::Value>, NodeExecutionError> {
    let mut deepseek_messages = Vec::new();

    for msg in messages {
      let deepseek_msg = serde_json::json!({
          "role": msg.role,
          "content": msg.content
      });
      deepseek_messages.push(deepseek_msg);
    }

    Ok(deepseek_messages)
  }

  /// Make HTTP request to DeepSeek API (placeholder for now)
  async fn call_deepseek_api(&self, messages: Vec<serde_json::Value>) -> Result<LLMResponse, NodeExecutionError> {
    let api_key = self.resolve_api_key()?;

    // For now, return a mock response
    // In a full implementation, this would make an actual HTTP request to the DeepSeek API
    log::info!("DeepSeek API call simulated with {} messages", messages.len());

    Ok(LLMResponse {
      content: format!("Mock DeepSeek response for model: {}", self.config.model),
      role: "assistant".to_string(),
      usage: Some(UsageStats { prompt_tokens: 100, completion_tokens: 50, total_tokens: 150 }),
    })
  }
}

#[async_trait]
impl SubNodeProvider for DeepSeekLLMProvider {
  fn provider_type(&self) -> SubNodeProviderType {
    SubNodeProviderType::LLM
  }

  fn get_node_definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.node_definition)
  }

  async fn initialize(&self) -> Result<(), NodeExecutionError> {
    // Validate API key availability
    self.resolve_api_key()?;

    // In a full implementation, we could make a test API call here
    // to validate the connection and credentials

    log::info!("DeepSeek LLM provider ({}) initialized successfully", self.config.model);
    Ok(())
  }
}

#[async_trait]
impl LLMSubNodeProvider for DeepSeekLLMProvider {
  async fn call_llm(&self, messages: Vec<Message>, config: LLMConfig) -> Result<LLMResponse, NodeExecutionError> {
    // Create a temporary config that merges provider config with request config
    let mut merged_config = self.config.clone();

    // Override with request-specific config
    if let Some(model) = Some(config.model.clone())
      && !model.is_empty()
      && model != "default"
    {
      merged_config.model = model;
    }
    if let Some(max_tokens) = config.max_tokens {
      merged_config.max_tokens = Some(max_tokens);
    }
    if let Some(temperature) = config.temperature {
      merged_config.temperature = Some(temperature);
    }
    if let Some(top_p) = config.top_p {
      merged_config.top_p = Some(top_p as f64);
    }
    if let Some(stop_sequences) = config.stop_sequences {
      merged_config.stop_sequences = Some(stop_sequences);
    }
    if let Some(api_key) = config.api_key {
      merged_config.api_key = Some(api_key);
    }

    // Convert messages to DeepSeek format
    let deepseek_messages = self.convert_messages_to_deepseek_format(messages)?;

    // Create a temporary provider with merged config for this call
    let temp_provider = DeepSeekLLMProvider {
      config: merged_config,
      node_definition: Arc::clone(&self.node_definition),
      provider_id: self.provider_id.clone(),
    };

    // Make the API call
    temp_provider.call_deepseek_api(deepseek_messages).await
  }
}

/// Factory function to create and configure DeepSeek LLM provider
pub fn create_deepseek_provider(
  config: Option<DeepSeekConfig>,
) -> Result<Arc<DeepSeekLLMProvider>, NodeExecutionError> {
  let config = config.unwrap_or_default();
  let provider = Arc::new(DeepSeekLLMProvider::new(config));
  Ok(provider)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deepseek_config_default() {
    let config = DeepSeekConfig::default();
    assert_eq!(config.model, "deepseek-chat");
    assert_eq!(config.max_tokens, Some(4096));
    assert_eq!(config.temperature, Some(0.7));
  }

  #[test]
  fn test_llm_config_conversion() {
    let llm_config = LLMConfig {
      model: "deepseek-coder".to_string(),
      max_tokens: Some(8000),
      temperature: Some(0.5),
      top_p: Some(90),
      stop_sequences: Some(vec!["```".to_string()]),
      api_key: Some("test-key".to_string()),
    };

    let deepseek_config = DeepSeekConfig::from(llm_config);
    assert_eq!(deepseek_config.model, "deepseek-coder");
    assert_eq!(deepseek_config.max_tokens, Some(8000));
    assert_eq!(deepseek_config.temperature, Some(0.5));
    assert_eq!(deepseek_config.top_p, Some(90.0));
    assert_eq!(deepseek_config.api_key, Some("test-key".to_string()));
  }

  #[test]
  fn test_provider_creation() {
    let config = DeepSeekConfig { model: "deepseek-coder".to_string(), ..Default::default() };

    let provider = DeepSeekLLMProvider::new(config);
    assert_eq!(provider.config().model, "deepseek-coder");
    assert!(!provider.provider_id().is_empty());
  }

  #[tokio::test]
  async fn test_provider_initialization() {
    // This test would require a valid API key to fully pass
    let config = DeepSeekConfig { api_key: Some("test-key".to_string()), ..Default::default() };

    let provider = DeepSeekLLMProvider::new(config);

    // Should succeed during initialization
    let result = provider.initialize().await;
    assert!(result.is_ok());
  }

  #[test]
  fn test_message_conversion() {
    let config = DeepSeekConfig::default();
    let provider = DeepSeekLLMProvider::new(config);

    let messages = vec![
      Message { role: "user".to_string(), content: "Hello, DeepSeek!".to_string() },
      Message { role: "assistant".to_string(), content: "Hello! How can I help you today?".to_string() },
    ];

    let result = provider.convert_messages_to_deepseek_format(messages);
    assert!(result.is_ok());

    let deepseek_messages = result.unwrap();
    assert_eq!(deepseek_messages.len(), 2);
    assert_eq!(deepseek_messages[0]["role"], "user");
    assert_eq!(deepseek_messages[0]["content"], "Hello, DeepSeek!");
    assert_eq!(deepseek_messages[1]["role"], "assistant");
    assert_eq!(deepseek_messages[1]["content"], "Hello! How can I help you today?");
  }

  #[tokio::test]
  async fn test_cluster_node_integration() {
    // Test the full integration with NodeRegistry and ClusterNodeExecutor
    use crate::workflow::{
      ClusterNodeConfig, NodeKind, NodeRegistry,
      graph_flow_tasks::{ClusterNodeExecutor, Context},
    };

    let config = DeepSeekConfig {
      model: "deepseek-chat".to_string(),
      api_key: Some("integration-test-key".to_string()),
      ..Default::default()
    };

    let provider = create_deepseek_provider(Some(config)).unwrap();
    provider.initialize().await.unwrap();

    let node_registry = NodeRegistry::new();
    let node_kind: NodeKind = "deepseek_llm".into();

    node_registry.register_subnode_provider(node_kind.clone(), provider.clone()).unwrap();
    assert!(node_registry.has_subnode_provider(&node_kind));

    let mut executor = ClusterNodeExecutor::new(node_registry);
    let cluster_config = ClusterNodeConfig {
      llm_config: Some(LLMConfig {
        model: "deepseek-chat".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        ..Default::default()
      }),
      ..Default::default()
    };

    executor.register_subnode_provider(node_kind, cluster_config).unwrap();
    assert_eq!(executor.task_count(), 1);

    let task_ids = executor.task_ids();
    let mut context = Context::new();
    context.set("test_key", "test_value").unwrap();

    let result = executor.execute_task(&task_ids[0], context).await.unwrap();
    assert!(result.response.is_some());

    println!("✅ Cluster Node integration test passed!");
  }
}
