//! Cluster Node Manager
//!
//! This module provides a unified management interface for the Cluster Node architecture,
//! enabling easy setup, configuration, and execution of multiple SubNodeProviders.

use serde_json::{Value, json};
use std::sync::Arc;

use crate::workflow::{
  NodeExecutionError, NodeKind, NodeRegistry,
  graph_flow_tasks::ClusterNodeExecutor,
  providers::{
    AiAgentProvider, AiAgentProviderConfig, DeepSeekConfig, DeepSeekLLMProvider, MemoryProvider, MemoryProviderConfig,
    create_ai_agent_provider, create_deepseek_provider, create_memory_provider,
  },
  sub_node_provider::{
    AgentConfig, ClusterNodeConfig, ExecutionConfig, LLMConfig, MemoryConfig, Message, SubNodeProvider,
    SubNodeProviderRef,
  },
};

/// Cluster Node configuration
#[derive(Debug, Clone)]
pub struct ClusterNodeManagerConfig {
  /// Whether to auto-register default providers
  pub auto_register_providers: bool,
  /// Default execution configuration
  pub default_execution_config: ExecutionConfig,
  /// DeepSeek configuration (if auto-registering)
  pub deepseek_config: Option<DeepSeekConfig>,
  /// Memory configuration (if auto-registering)
  pub memory_config: Option<MemoryProviderConfig>,
  /// AI Agent configuration (if auto-registering)
  pub agent_config: Option<AiAgentProviderConfig>,
}

impl Default for ClusterNodeManagerConfig {
  fn default() -> Self {
    Self {
      auto_register_providers: true,
      default_execution_config: ExecutionConfig::default(),
      deepseek_config: Some(DeepSeekConfig::default()),
      memory_config: Some(MemoryProviderConfig::default()),
      agent_config: Some(AiAgentProviderConfig::default()),
    }
  }
}

/// Unified Cluster Node Manager
pub struct ClusterNodeManager {
  registry: Arc<NodeRegistry>,
  executor: ClusterNodeExecutor,
  config: ClusterNodeManagerConfig,
}

impl std::fmt::Debug for ClusterNodeManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ClusterNodeManager")
      .field("registry", &"Arc<NodeRegistry>")
      .field("executor", &"ClusterNodeExecutor")
      .field("config", &self.config)
      .finish()
  }
}

impl ClusterNodeManager {
  /// Create a new Cluster Node Manager
  pub fn new(config: ClusterNodeManagerConfig) -> Self {
    let registry = Arc::new(NodeRegistry::new());
    let executor = ClusterNodeExecutor::new_with_shared_registry(registry.clone());

    Self { registry, executor, config }
  }

  /// Create a manager with default configuration
  pub fn new_default() -> Self {
    Self::new(ClusterNodeManagerConfig::default())
  }

  /// Initialize the manager with default providers
  pub async fn initialize(&mut self) -> Result<(), NodeExecutionError> {
    if self.config.auto_register_providers {
      self.register_default_providers().await?;
    }
    Ok(())
  }

  /// Register default providers
  async fn register_default_providers(&mut self) -> Result<(), NodeExecutionError> {
    log::info!("Registering default SubNodeProviders...");

    // Register DeepSeek LLM Provider
    if let Some(deepseek_config) = &self.config.deepseek_config {
      let provider = create_deepseek_provider(Some(deepseek_config.clone()))?;
      provider.initialize().await?;

      let kind: NodeKind = "deepseek_llm".into();
      self
        .registry
        .register_subnode_provider(kind.clone(), provider)
        .map_err(|e| NodeExecutionError::ConfigurationError(format!("Registration failed: {}", e)))?;

      let cluster_config = ClusterNodeConfig {
        llm_config: Some(LLMConfig {
          model: deepseek_config.model.clone(),
          max_tokens: deepseek_config.max_tokens,
          temperature: deepseek_config.temperature,
          top_p: deepseek_config.top_p.map(|v| v as u32),
          stop_sequences: deepseek_config.stop_sequences.clone(),
          api_key: deepseek_config.api_key.clone(),
        }),
        execution_config: self.config.default_execution_config.clone(),
        ..Default::default()
      };
      self.executor.register_subnode_provider(kind, cluster_config)?;

      log::info!("DeepSeek LLM Provider registered");
    }

    // Register Memory Provider
    if let Some(memory_config) = &self.config.memory_config {
      let provider = create_memory_provider(Some(memory_config.clone()))?;
      provider.initialize().await?;

      let kind: NodeKind = "memory_provider".into();
      self
        .registry
        .register_subnode_provider(kind.clone(), provider)
        .map_err(|e| NodeExecutionError::ConfigurationError(format!("Registration failed: {}", e)))?;

      let cluster_config = ClusterNodeConfig {
        memory_config: Some(MemoryConfig {
          context_window: Some(memory_config.max_messages),
          max_history: Some(memory_config.max_messages),
          persistence_enabled: Some(memory_config.persistence_enabled),
        }),
        execution_config: self.config.default_execution_config.clone(),
        ..Default::default()
      };
      self.executor.register_subnode_provider(kind, cluster_config)?;

      log::info!("Memory Provider registered");
    }

    // Register AI Agent Provider
    if let Some(agent_config) = &self.config.agent_config {
      let provider = create_ai_agent_provider(Some(agent_config.clone()))?;
      provider.initialize().await?;

      let kind: NodeKind = "ai_agent_provider".into();
      self
        .registry
        .register_subnode_provider(kind.clone(), provider)
        .map_err(|e| NodeExecutionError::ConfigurationError(format!("Registration failed: {}", e)))?;

      let cluster_config = ClusterNodeConfig {
        agent_config: Some(AgentConfig {
          system_prompt: Some(agent_config.default_system_prompt.clone()),
          max_iterations: Some(agent_config.max_iterations),
          temperature: Some(agent_config.default_temperature),
          enable_streaming: Some(agent_config.enable_streaming),
          enable_tools: Some(agent_config.enable_tools),
          session_id: None,
        }),
        execution_config: self.config.default_execution_config.clone(),
        ..Default::default()
      };
      self.executor.register_subnode_provider(kind, cluster_config)?;

      log::info!("AI Agent Provider registered");
    }

    Ok(())
  }

  /// Register a custom SubNodeProvider
  pub async fn register_provider<P>(
    &mut self,
    kind: NodeKind,
    provider: Arc<P>,
    cluster_config: ClusterNodeConfig,
  ) -> Result<(), NodeExecutionError>
  where
    P: crate::workflow::sub_node_provider::SubNodeProvider + 'static,
  {
    provider.initialize().await?;
    self
      .registry
      .register_subnode_provider(kind.clone(), provider)
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("Registration failed: {}", e)))?;
    self.executor.register_subnode_provider(kind, cluster_config)?;
    Ok(())
  }

  /// Execute a task by provider kind
  pub async fn execute_task(&self, provider_kind: &str, input: Value) -> Result<String, NodeExecutionError> {
    let task_ids = self.executor.task_ids();

    // Find the task for the requested provider
    let target_task_id = task_ids.iter().find(|id| id.contains(provider_kind)).ok_or_else(|| {
      NodeExecutionError::ConfigurationError(format!("No task found for provider: {}", provider_kind))
    })?;

    let mut context = self.executor.create_context();
    context.set("input", input)?;

    let result = self.executor.execute_task(target_task_id, context).await?;

    result
      .response
      .ok_or_else(|| NodeExecutionError::InvalidInput("Task returned no response".to_string()))
  }

  /// Get manager statistics
  pub fn get_statistics(&self) -> ClusterNodeStatistics {
    ClusterNodeStatistics {
      registered_providers: self.registry.subnode_provider_count(),
      active_tasks: self.executor.task_count(),
      provider_types: self.get_registered_provider_types(),
    }
  }

  /// Get registered provider types
  fn get_registered_provider_types(&self) -> Vec<String> {
    let task_ids = self.executor.task_ids();
    task_ids
      .iter()
      .filter_map(|id| {
        if id.contains("deepseek") {
          Some("LLM".to_string())
        } else if id.contains("memory") {
          Some("Memory".to_string())
        } else if id.contains("agent") {
          Some("Agent".to_string())
        } else {
          None
        }
      })
      .collect()
  }

  /// Get the internal registry (for advanced usage)
  pub fn registry(&self) -> &Arc<NodeRegistry> {
    &self.registry
  }

  /// Get the internal executor (for advanced usage)
  pub fn executor(&self) -> &ClusterNodeExecutor {
    &self.executor
  }
}

/// Cluster Node statistics
#[derive(Debug, Clone)]
pub struct ClusterNodeStatistics {
  pub registered_providers: usize,
  pub active_tasks: usize,
  pub provider_types: Vec<String>,
}

/// Builder for ClusterNodeManager
pub struct ClusterNodeManagerBuilder {
  config: ClusterNodeManagerConfig,
}

impl ClusterNodeManagerBuilder {
  /// Create a new builder
  pub fn new() -> Self {
    Self { config: ClusterNodeManagerConfig::default() }
  }

  /// Disable auto-registration of default providers
  pub fn without_default_providers(mut self) -> Self {
    self.config.auto_register_providers = false;
    self
  }

  /// Set DeepSeek configuration
  pub fn with_deepseek_config(mut self, config: DeepSeekConfig) -> Self {
    self.config.deepseek_config = Some(config);
    self
  }

  /// Set Memory configuration
  pub fn with_memory_config(mut self, config: MemoryProviderConfig) -> Self {
    self.config.memory_config = Some(config);
    self
  }

  /// Set AI Agent configuration
  pub fn with_agent_config(mut self, config: AiAgentProviderConfig) -> Self {
    self.config.agent_config = Some(config);
    self
  }

  /// Set default execution configuration
  pub fn with_execution_config(mut self, config: ExecutionConfig) -> Self {
    self.config.default_execution_config = config;
    self
  }

  /// Build the manager
  pub fn build(self) -> ClusterNodeManager {
    ClusterNodeManager::new(self.config)
  }
}

impl Default for ClusterNodeManagerBuilder {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_manager_creation() {
    let manager = ClusterNodeManager::new_default();
    let stats = manager.get_statistics();

    // Before initialization, should have no providers
    assert_eq!(stats.registered_providers, 0);
    assert_eq!(stats.active_tasks, 0);
  }

  #[tokio::test]
  async fn test_manager_initialization() {
    let mut manager = ClusterNodeManager::new_default();
    manager.initialize().await.unwrap();

    let stats = manager.get_statistics();
    assert_eq!(stats.registered_providers, 3);
    assert_eq!(stats.active_tasks, 3);
    assert!(stats.provider_types.contains(&"LLM".to_string()));
    assert!(stats.provider_types.contains(&"Memory".to_string()));
    assert!(stats.provider_types.contains(&"Agent".to_string()));
  }

  #[tokio::test]
  async fn test_builder_pattern() {
    let manager = ClusterNodeManagerBuilder::new()
      .without_default_providers()
      .with_execution_config(ExecutionConfig {
        timeout_seconds: Some(120),
        max_retries: Some(5),
        parallel_execution: Some(false),
      })
      .build();

    let mut manager = manager;
    manager.initialize().await.unwrap();

    let stats = manager.get_statistics();
    assert_eq!(stats.registered_providers, 0);
    assert_eq!(stats.active_tasks, 0);
  }

  #[tokio::test]
  async fn test_custom_providers() {
    let mut manager = ClusterNodeManagerBuilder::new().without_default_providers().build();

    // Register a custom DeepSeek provider
    let deepseek_config = DeepSeekConfig {
      model: "custom-model".to_string(),
      api_key: Some("custom-key".to_string()),
      ..Default::default()
    };

    let provider = create_deepseek_provider(Some(deepseek_config.clone())).unwrap();
    provider.initialize().await.unwrap();

    let kind: NodeKind = "custom_deepseek".into();
    let cluster_config = ClusterNodeConfig { llm_config: Some(LLMConfig::from(deepseek_config)), ..Default::default() };

    manager.register_provider(kind, provider, cluster_config).await.unwrap();

    let stats = manager.get_statistics();
    assert_eq!(stats.registered_providers, 1);
    assert_eq!(stats.active_tasks, 1);
  }
}
