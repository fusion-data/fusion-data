//! AI Agent SubNodeProvider Implementation
//!
//! This module provides a SubNodeProvider implementation for AI Agent functionality,
//! enabling intelligent orchestration of LLM, Memory, and Tool interactions within
//! the Cluster Node architecture.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid;

use crate::workflow::{
  NodeDefinition, NodeExecutionError,
  sub_node_provider::{
    AgentConfig, AgentResponse, AgentSubNodeProvider, AgentUsageStats, LLMConfig, Message, SessionInfo,
    SubNodeProvider, SubNodeProviderType, ToolCallRequest,
  },
  workflow_node::NodeGroupKind,
};

use crate::version::Version;

/// Agent execution state
#[derive(Debug, Clone)]
struct AgentExecutionState {
  current_iteration: u32,
  total_llm_calls: u32,
  total_tool_calls: u32,
  start_time: DateTime<Utc>,
  session_id: Option<String>,
}

impl AgentExecutionState {
  fn new(session_id: Option<String>) -> Self {
    Self { current_iteration: 0, total_llm_calls: 0, total_tool_calls: 0, start_time: Utc::now(), session_id }
  }

  fn should_continue(&self, max_iterations: Option<u32>) -> bool {
    if let Some(max_iter) = max_iterations {
      self.current_iteration < max_iter
    } else {
      true // Default: continue until completion
    }
  }

  fn increment_iteration(&mut self) {
    self.current_iteration += 1;
  }

  fn increment_llm_calls(&mut self) {
    self.total_llm_calls += 1;
  }

  fn increment_tool_calls(&mut self) {
    self.total_tool_calls += 1;
  }

  fn create_usage_stats(&self) -> AgentUsageStats {
    AgentUsageStats {
      total_iterations: self.current_iteration,
      llm_calls: self.total_llm_calls,
      tool_calls: self.total_tool_calls,
      total_tokens: None, // Will be filled by LLM provider if available
    }
  }
}

/// AI Agent-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgentProviderConfig {
  /// Default system prompt
  pub default_system_prompt: String,
  /// Maximum iterations
  pub max_iterations: u32,
  /// Default temperature
  pub default_temperature: f64,
  /// Enable streaming by default
  pub enable_streaming: bool,
  /// Enable tools by default
  pub enable_tools: bool,
  /// Session timeout in seconds
  pub session_timeout_seconds: u64,
}

impl Default for AiAgentProviderConfig {
  fn default() -> Self {
    Self {
      default_system_prompt: "You are a helpful AI assistant with access to various tools and memory.".to_string(),
      max_iterations: 10,
      default_temperature: 0.7,
      enable_streaming: false,
      enable_tools: true,
      session_timeout_seconds: 3600, // 1 hour
    }
  }
}

impl From<AgentConfig> for AiAgentProviderConfig {
  fn from(agent_config: AgentConfig) -> Self {
    Self {
      default_system_prompt: agent_config
        .system_prompt
        .unwrap_or_else(|| "You are a helpful AI assistant.".to_string()),
      max_iterations: agent_config.max_iterations.unwrap_or(10),
      default_temperature: agent_config.temperature.unwrap_or(0.7),
      enable_streaming: agent_config.enable_streaming.unwrap_or(false),
      enable_tools: agent_config.enable_tools.unwrap_or(true),
      session_timeout_seconds: 3600,
    }
  }
}

/// AI Agent SubNodeProvider implementation
#[derive(Clone)]
pub struct AiAgentProvider {
  config: AiAgentProviderConfig,
  node_definition: Arc<NodeDefinition>,
  provider_id: String,
  // In a real implementation, these would be references to actual providers
  // For now, we'll use placeholder structures
  llm_provider: Option<Arc<dyn crate::workflow::sub_node_provider::LLMSubNodeProvider>>,
  memory_provider: Option<Arc<dyn crate::workflow::sub_node_provider::MemorySubNodeProvider>>,
}

impl std::fmt::Debug for AiAgentProvider {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AiAgentProvider")
      .field("config", &self.config)
      .field("provider_id", &self.provider_id)
      .field("has_llm_provider", &self.llm_provider.is_some())
      .field("has_memory_provider", &self.memory_provider.is_some())
      .finish()
  }
}

impl AiAgentProvider {
  /// Create a new AI Agent provider
  pub fn new(config: AiAgentProviderConfig) -> Self {
    let provider_id = format!("ai_agent_{}", uuid::Uuid::now_v7());

    // Create node definition for AI Agent provider
    let node_definition = Arc::new(NodeDefinition {
      kind: "ai_agent_provider".into(),
      version: Version::parse("1.0.0").unwrap(),
      groups: vec![NodeGroupKind::Transform],
      display_name: "AI Agent Provider".to_string(),
      description: Some("Intelligent AI agent orchestrator with LLM, memory, and tool integration".to_string()),
      inputs: vec![],
      outputs: vec![],
      properties: vec![],
      document_url: Some("https://docs.hetumind.ai/ai-agent".to_string()),
      sub_title: Some("Intelligent task orchestration".to_string()),
      hidden: false,
      max_nodes: None,
      icon: Some("robot".to_string()),
      icon_color: Some(crate::types::IconColor::Purple),
      icon_url: None,
      badge_icon_url: None,
    });

    Self { config, node_definition, provider_id, llm_provider: None, memory_provider: None }
  }

  /// Create provider from AgentConfig
  pub fn from_agent_config(agent_config: AgentConfig) -> Self {
    Self::new(AiAgentProviderConfig::from(agent_config))
  }

  /// Set LLM provider
  pub fn with_llm_provider(
    mut self,
    provider: Arc<dyn crate::workflow::sub_node_provider::LLMSubNodeProvider>,
  ) -> Self {
    self.llm_provider = Some(provider);
    self
  }

  /// Set Memory provider
  pub fn with_memory_provider(
    mut self,
    provider: Arc<dyn crate::workflow::sub_node_provider::MemorySubNodeProvider>,
  ) -> Self {
    self.memory_provider = Some(provider);
    self
  }

  /// Get the provider's unique ID
  pub fn provider_id(&self) -> &str {
    &self.provider_id
  }

  /// Get current configuration
  pub fn config(&self) -> &AiAgentProviderConfig {
    &self.config
  }

  /// Update configuration
  pub fn update_config(&mut self, config: AiAgentProviderConfig) {
    self.config = config;
  }

  /// Resolve session ID from configuration or generate new one
  fn resolve_session_id(&self, config: &AgentConfig) -> String {
    config.session_id.clone().unwrap_or_else(|| format!("agent_session_{}", uuid::Uuid::now_v7()))
  }

  /// Prepare system prompt based on configuration
  fn prepare_system_prompt(&self, config: &AgentConfig) -> String {
    config.system_prompt.clone().unwrap_or_else(|| self.config.default_system_prompt.clone())
  }

  /// Retrieve conversation history from memory
  async fn retrieve_conversation_history(
    &self,
    session_id: &str,
    context_window: Option<usize>,
  ) -> Result<Vec<Message>, NodeExecutionError> {
    if let Some(memory_provider) = &self.memory_provider {
      let count = context_window.unwrap_or(10);
      memory_provider.retrieve_messages(session_id, count).await
    } else {
      Ok(Vec::new())
    }
  }

  /// Store messages to memory
  async fn store_conversation_messages(
    &self,
    session_id: &str,
    messages: Vec<Message>,
  ) -> Result<(), NodeExecutionError> {
    if let Some(memory_provider) = &self.memory_provider {
      memory_provider.store_messages(session_id, messages).await?;
    }
    Ok(())
  }

  /// Execute LLM call
  async fn call_llm(&self, messages: Vec<Message>, config: &AgentConfig) -> Result<String, NodeExecutionError> {
    if let Some(llm_provider) = &self.llm_provider {
      let llm_config = LLMConfig {
        model: "default".to_string(), // Would be configurable in real implementation
        max_tokens: None,
        temperature: config.temperature.or(Some(self.config.default_temperature)),
        top_p: None,
        stop_sequences: None,
        api_key: None,
      };

      let response = llm_provider.call_llm(messages, llm_config).await?;
      Ok(response.content)
    } else {
      // Mock response for testing
      Ok(
        "This is a mock AI agent response. In a real implementation, this would be generated by an LLM provider."
          .to_string(),
      )
    }
  }

  /// Execute agent iteration
  async fn execute_iteration(
    &self,
    messages: Vec<Message>,
    config: &AgentConfig,
    state: &mut AgentExecutionState,
  ) -> Result<(String, Option<Vec<ToolCallRequest>>), NodeExecutionError> {
    state.increment_iteration();
    state.increment_llm_calls();

    // Call LLM
    let response = self.call_llm(messages.clone(), config).await?;

    // In a real implementation, we would parse the response for tool calls
    // For now, we'll return the response without tool calls
    Ok((response, None))
  }

  /// Create session info
  fn create_session_info(&self, state: &AgentExecutionState, history_length: usize) -> SessionInfo {
    SessionInfo {
      session_id: state.session_id.clone().unwrap_or_default(),
      history_length,
      has_memory: self.memory_provider.is_some(),
    }
  }
}

#[async_trait]
impl SubNodeProvider for AiAgentProvider {
  fn provider_type(&self) -> SubNodeProviderType {
    SubNodeProviderType::Agent
  }

  fn get_node_definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.node_definition)
  }

  async fn initialize(&self) -> Result<(), NodeExecutionError> {
    log::info!(
      "AI Agent provider ({}) initialized with max_iterations: {}, enable_tools: {}",
      self.provider_id,
      self.config.max_iterations,
      self.config.enable_tools
    );

    // Initialize sub-providers if available
    if let Some(llm_provider) = &self.llm_provider {
      llm_provider.initialize().await?;
    }
    if let Some(memory_provider) = &self.memory_provider {
      memory_provider.initialize().await?;
    }

    Ok(())
  }
}

#[async_trait]
impl AgentSubNodeProvider for AiAgentProvider {
  async fn execute_agent(
    &self,
    messages: Vec<Message>,
    config: AgentConfig,
  ) -> Result<AgentResponse, NodeExecutionError> {
    let session_id = self.resolve_session_id(&config);
    let mut state = AgentExecutionState::new(Some(session_id.clone()));

    log::info!("Executing AI Agent with session_id: {}, {} input messages", session_id, messages.len());

    // Prepare system prompt
    let system_prompt = self.prepare_system_prompt(&config);

    // Retrieve conversation history
    let history_messages = self
      .retrieve_conversation_history(
        &session_id,
        Some(5), // TODO: Make configurable
      )
      .await?;

    log::info!("Retrieved {} messages from history", history_messages.len());

    // Combine system prompt, history, and current messages
    let mut all_messages = Vec::new();

    // Add system message
    if !system_prompt.is_empty() {
      all_messages.push(Message { role: "system".to_string(), content: system_prompt });
    }

    // Add history messages
    all_messages.extend(history_messages);

    // Add current messages
    all_messages.extend(messages.clone());

    // Execute agent iterations
    let mut final_response = String::new();
    let mut tool_calls = None;

    while state.should_continue(config.max_iterations) {
      log::debug!("Agent iteration {}/{}", state.current_iteration + 1, config.max_iterations.unwrap_or(999));

      match self.execute_iteration(all_messages.clone(), &config, &mut state).await {
        Ok((response, calls)) => {
          final_response = response.clone();
          tool_calls = calls;

          // Add assistant response to conversation
          all_messages.push(Message { role: "assistant".to_string(), content: response });

          // In a real implementation, we would:
          // 1. Check if tool calls are present
          // 2. Execute tool calls if present
          // 3. Add tool results back to conversation
          // 4. Continue iteration if needed

          // For now, we'll break after first iteration
          break;
        }
        Err(e) => {
          log::error!("Agent iteration failed: {}", e);
          return Err(e);
        }
      }
    }

    // Store conversation to memory
    let mut messages_to_store = messages;
    messages_to_store.push(Message { role: "assistant".to_string(), content: final_response.clone() });

    self.store_conversation_messages(&session_id, messages_to_store).await?;

    // Create response
    let usage_stats = state.create_usage_stats();
    let session_info = self.create_session_info(&state, all_messages.len() - 1); // Exclude system message

    log::info!(
      "Agent execution completed: {} iterations, {} LLM calls",
      usage_stats.total_iterations,
      usage_stats.llm_calls
    );

    Ok(AgentResponse {
      content: final_response,
      role: "assistant".to_string(),
      tool_calls,
      usage: Some(usage_stats),
      session_info: Some(session_info),
    })
  }
}

/// Factory function to create and configure AI Agent provider
pub fn create_ai_agent_provider(
  config: Option<AiAgentProviderConfig>,
) -> Result<Arc<AiAgentProvider>, NodeExecutionError> {
  let config = config.unwrap_or_default();
  let provider = Arc::new(AiAgentProvider::new(config));
  Ok(provider)
}

/// Create AI Agent provider from AgentConfig
pub fn create_ai_agent_provider_from_config(config: AgentConfig) -> Result<Arc<AiAgentProvider>, NodeExecutionError> {
  let provider = Arc::new(AiAgentProvider::from_agent_config(config));
  Ok(provider)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ai_agent_provider_config_default() {
    let config = AiAgentProviderConfig::default();
    assert_eq!(config.max_iterations, 10);
    assert_eq!(config.default_temperature, 0.7);
    assert!(!config.enable_streaming);
    assert!(config.enable_tools);
  }

  #[test]
  fn test_agent_config_conversion() {
    let agent_config = AgentConfig {
      system_prompt: Some("Custom system prompt".to_string()),
      max_iterations: Some(20),
      temperature: Some(0.5),
      enable_streaming: Some(true),
      enable_tools: Some(false),
      session_id: Some("test_session".to_string()),
    };

    let provider_config = AiAgentProviderConfig::from(agent_config);
    assert_eq!(provider_config.default_system_prompt, "Custom system prompt");
    assert_eq!(provider_config.max_iterations, 20);
    assert_eq!(provider_config.default_temperature, 0.5);
    assert!(provider_config.enable_streaming);
    assert!(!provider_config.enable_tools);
  }

  #[test]
  fn test_agent_execution_state() {
    let mut state = AgentExecutionState::new(Some("test_session".to_string()));

    assert!(state.should_continue(Some(5)));
    assert_eq!(state.current_iteration, 0);
    assert_eq!(state.total_llm_calls, 0);

    state.increment_iteration();
    assert_eq!(state.current_iteration, 1);

    state.increment_llm_calls();
    assert_eq!(state.total_llm_calls, 1);

    assert!(state.should_continue(Some(5)));

    // Simulate reaching max iterations
    for _ in 0..4 {
      state.increment_iteration();
    }
    assert_eq!(state.current_iteration, 5);
    assert!(!state.should_continue(Some(5)));
  }

  #[test]
  fn test_provider_creation() {
    let config = AiAgentProviderConfig { max_iterations: 15, enable_streaming: true, ..Default::default() };

    let provider = AiAgentProvider::new(config);
    assert_eq!(provider.config().max_iterations, 15);
    assert!(provider.config().enable_streaming);
    assert!(!provider.provider_id().is_empty());
  }

  #[tokio::test]
  async fn test_provider_initialization() {
    let config = AiAgentProviderConfig::default();
    let provider = AiAgentProvider::new(config);

    let result = provider.initialize().await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_agent_execution() {
    let config = AiAgentProviderConfig::default();
    let provider = AiAgentProvider::new(config);

    // Initialize provider
    provider.initialize().await.unwrap();

    let agent_config = AgentConfig {
      system_prompt: Some("You are a helpful assistant.".to_string()),
      max_iterations: Some(3),
      ..Default::default()
    };

    let messages = vec![Message { role: "user".to_string(), content: "Hello, AI Agent!".to_string() }];

    // Execute agent
    let response = provider.execute_agent(messages, agent_config).await.unwrap();

    assert!(!response.content.is_empty());
    assert_eq!(response.role, "assistant");
    assert!(response.usage.is_some());
    assert!(response.session_info.is_some());

    let usage = response.usage.unwrap();
    assert_eq!(usage.total_iterations, 1);
    assert_eq!(usage.llm_calls, 1);
  }

  #[tokio::test]
  async fn test_factory_function() {
    let config = Some(AiAgentProviderConfig { max_iterations: 25, enable_tools: false, ..Default::default() });

    let provider = create_ai_agent_provider(config).unwrap();
    assert_eq!(provider.config().max_iterations, 25);
    assert!(!provider.config().enable_tools);

    provider.initialize().await.unwrap();
  }

  #[tokio::test]
  async fn test_agent_config_factory() {
    let agent_config = AgentConfig {
      system_prompt: Some("Test system prompt".to_string()),
      max_iterations: Some(5),
      temperature: Some(0.2),
      enable_streaming: Some(true),
      session_id: Some("factory_test_session".to_string()),
      ..Default::default()
    };

    let provider = create_ai_agent_provider_from_config(agent_config).unwrap();
    assert_eq!(provider.config().default_system_prompt, "Test system prompt");
    assert_eq!(provider.config().max_iterations, 5);
    assert_eq!(provider.config().default_temperature, 0.2);
    assert!(provider.config().enable_streaming);

    provider.initialize().await.unwrap();
  }

  #[tokio::test]
  async fn test_cluster_node_integration() {
    // Test the full integration with NodeRegistry and ClusterNodeExecutor
    use crate::workflow::{
      ClusterNodeConfig, ExecutionConfig, NodeKind, NodeRegistry,
      graph_flow_tasks::{ClusterNodeExecutor, Context},
    };

    let config = AiAgentProviderConfig { max_iterations: 5, enable_tools: true, ..Default::default() };

    let provider = create_ai_agent_provider(Some(config)).unwrap();
    provider.initialize().await.unwrap();

    let node_registry = NodeRegistry::new();
    let node_kind: NodeKind = "ai_agent_provider".into();

    node_registry.register_subnode_provider(node_kind.clone(), provider.clone()).unwrap();
    assert!(node_registry.has_subnode_provider(&node_kind));

    let mut executor = ClusterNodeExecutor::new(node_registry);
    let cluster_config = ClusterNodeConfig {
      agent_config: Some(AgentConfig {
        system_prompt: Some("You are a test AI agent.".to_string()),
        max_iterations: Some(3),
        temperature: Some(0.8),
        ..Default::default()
      }),
      ..Default::default()
    };

    executor.register_subnode_provider(node_kind, cluster_config).unwrap();
    assert_eq!(executor.task_count(), 1);

    let task_ids = executor.task_ids();
    let mut context = Context::new();
    context.set("test_input", "Hello from integration test").unwrap();

    let result = executor.execute_task(&task_ids[0], context).await.unwrap();
    assert!(result.response.is_some());

    println!("✅ AI Agent Provider Cluster Node integration test passed!");
  }
}
