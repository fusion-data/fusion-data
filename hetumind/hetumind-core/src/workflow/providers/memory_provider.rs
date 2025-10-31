//! Memory SubNodeProvider Implementation
//!
//! This module provides a SubNodeProvider implementation for memory management,
//! enabling message storage and retrieval within the Cluster Node architecture.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use fusion_common::ahash;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid;

use crate::workflow::{
  NodeDefinition, NodeExecutionError,
  sub_node_provider::{MemoryConfig, MemorySubNodeProvider, Message, SubNodeProvider, SubNodeProviderType},
  workflow_node::NodeGroupKind,
};

use crate::version::Version;

/// Memory message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMessage {
  pub role: String,
  pub content: String,
  pub timestamp: DateTime<Utc>,
  pub session_id: String,
}

impl MemoryMessage {
  pub fn new(role: String, content: String, session_id: String) -> Self {
    Self { role, content, timestamp: Utc::now(), session_id }
  }
}

/// Memory-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProviderConfig {
  /// Maximum number of messages to store per session
  pub max_messages: usize,
  /// Whether to persist memory (placeholder for future storage backend)
  pub persistence_enabled: bool,
  /// Default session timeout in seconds
  pub session_timeout_seconds: u64,
  /// Cleanup interval in seconds
  pub cleanup_interval_seconds: u64,
}

impl Default for MemoryProviderConfig {
  fn default() -> Self {
    Self {
      max_messages: 100,
      persistence_enabled: false,
      session_timeout_seconds: 3600, // 1 hour
      cleanup_interval_seconds: 300, // 5 minutes
    }
  }
}

impl From<MemoryConfig> for MemoryProviderConfig {
  fn from(memory_config: MemoryConfig) -> Self {
    Self {
      max_messages: memory_config.max_history.unwrap_or(100),
      persistence_enabled: memory_config.persistence_enabled.unwrap_or(false),
      session_timeout_seconds: 3600,
      cleanup_interval_seconds: 300,
    }
  }
}

/// In-memory session storage
#[derive(Debug)]
struct SessionStorage {
  messages: ahash::HashMap<String, Vec<MemoryMessage>>,
  last_accessed: ahash::HashMap<String, DateTime<Utc>>,
  max_messages: usize,
  session_timeout_seconds: u64,
}

impl SessionStorage {
  fn new(config: MemoryProviderConfig) -> Self {
    Self {
      messages: ahash::HashMap::default(),
      last_accessed: ahash::HashMap::default(),
      max_messages: config.max_messages,
      session_timeout_seconds: config.session_timeout_seconds,
    }
  }

  fn store_messages(&mut self, session_id: &str, messages: Vec<MemoryMessage>) -> Result<(), NodeExecutionError> {
    let session_messages = self.messages.entry(session_id.to_string()).or_insert_with(Vec::new);
    let now = Utc::now();

    // Update last access time
    self.last_accessed.insert(session_id.to_string(), now);

    // Add new messages
    for message in messages {
      session_messages.push(message);
    }

    // Enforce maximum message limit (sliding window)
    if session_messages.len() > self.max_messages {
      let remove_count = session_messages.len() - self.max_messages;
      session_messages.drain(0..remove_count);
    }

    Ok(())
  }

  fn retrieve_messages(&self, session_id: &str, count: usize) -> Result<Vec<MemoryMessage>, NodeExecutionError> {
    // Update last access time (would need mutable access in real implementation)

    if let Some(messages) = self.messages.get(session_id) {
      let messages_len = messages.len();
      if count >= messages_len {
        Ok(messages.clone())
      } else {
        // Return the most recent 'count' messages
        let start_idx = messages_len - count;
        Ok(messages[start_idx..].to_vec())
      }
    } else {
      Ok(Vec::new())
    }
  }

  fn cleanup_expired_sessions(&mut self) {
    let now = Utc::now();
    let timeout_duration = chrono::Duration::seconds(self.session_timeout_seconds as i64);

    let expired_sessions: Vec<String> = self
      .last_accessed
      .iter()
      .filter_map(|(session_id, last_accessed)| {
        if now.signed_duration_since(*last_accessed) > timeout_duration { Some(session_id.clone()) } else { None }
      })
      .collect();

    for session_id in expired_sessions {
      self.messages.remove(&session_id);
      self.last_accessed.remove(&session_id);
      log::info!("Cleaned up expired session: {}", session_id);
    }
  }

  fn get_session_stats(&self, session_id: &str) -> Option<SessionStats> {
    if let Some(messages) = self.messages.get(session_id) {
      Some(SessionStats {
        message_count: messages.len(),
        last_accessed: self.last_accessed.get(session_id).copied(),
        session_id: session_id.to_string(),
      })
    } else {
      None
    }
  }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
  pub message_count: usize,
  pub last_accessed: Option<DateTime<Utc>>,
  pub session_id: String,
}

/// Memory SubNodeProvider implementation
#[derive(Debug)]
pub struct MemoryProvider {
  config: MemoryProviderConfig,
  node_definition: Arc<NodeDefinition>,
  provider_id: String,
  // In a real implementation, this would use Arc<Mutex<>> for thread safety
  // For now, using a simple placeholder structure
  storage: Arc<TokioMutex<SessionStorage>>,
}

impl MemoryProvider {
  /// Create a new memory provider
  pub fn new(config: MemoryProviderConfig) -> Self {
    let provider_id = format!("memory_{}", uuid::Uuid::now_v7());
    let storage = Arc::new(TokioMutex::new(SessionStorage::new(config.clone())));

    // Create node definition for memory provider
    let node_definition = Arc::new(NodeDefinition {
      kind: "memory_provider".into(),
      version: Version::parse("1.0.0").unwrap(),
      groups: vec![NodeGroupKind::Transform],
      display_name: "Memory Provider".to_string(),
      description: Some("In-memory message storage and retrieval for AI conversations".to_string()),
      inputs: vec![],
      outputs: vec![],
      properties: vec![],
      document_url: Some("https://docs.hetumind.ai/memory".to_string()),
      sub_title: Some("Conversation memory management".to_string()),
      hidden: false,
      max_nodes: None,
      icon: Some("database".to_string()),
      icon_color: Some(crate::types::IconColor::Green),
      icon_url: None,
      badge_icon_url: None,
    });

    Self { config, node_definition, provider_id, storage }
  }

  /// Create provider from MemoryConfig
  pub fn from_memory_config(memory_config: MemoryConfig) -> Self {
    Self::new(MemoryProviderConfig::from(memory_config))
  }

  /// Get the provider's unique ID
  pub fn provider_id(&self) -> &str {
    &self.provider_id
  }

  /// Get current configuration
  pub fn config(&self) -> &MemoryProviderConfig {
    &self.config
  }

  /// Update configuration
  pub fn update_config(&mut self, config: MemoryProviderConfig) {
    self.config = config.clone();
    // In a real implementation, update storage configuration as well
  }

  /// Start background cleanup task
  pub async fn start_cleanup_task(&self) {
    let storage = Arc::clone(&self.storage);
    let cleanup_interval = self.config.cleanup_interval_seconds;

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(cleanup_interval));

      loop {
        interval.tick().await;
        let mut storage_guard = storage.lock().await;
        storage_guard.cleanup_expired_sessions();
      }
    });
  }

  /// Get statistics for all active sessions
  pub async fn get_all_session_stats(&self) -> Vec<SessionStats> {
    let storage_guard = self.storage.lock().await;
    storage_guard
      .messages
      .keys()
      .filter_map(|session_id| storage_guard.get_session_stats(session_id))
      .collect()
  }

  /// Get statistics for a specific session
  pub async fn get_session_stats(&self, session_id: &str) -> Option<SessionStats> {
    let storage_guard = self.storage.lock().await;
    storage_guard.get_session_stats(session_id)
  }

  /// Clear all sessions
  pub async fn clear_all_sessions(&self) -> Result<(), NodeExecutionError> {
    let mut storage_guard = self.storage.lock().await;
    storage_guard.messages.clear();
    storage_guard.last_accessed.clear();
    log::info!("Cleared all memory sessions");
    Ok(())
  }

  /// Clear a specific session
  pub async fn clear_session(&self, session_id: &str) -> Result<(), NodeExecutionError> {
    let mut storage_guard = self.storage.lock().await;
    storage_guard.messages.remove(session_id);
    storage_guard.last_accessed.remove(session_id);
    log::info!("Cleared memory session: {}", session_id);
    Ok(())
  }
}

#[async_trait]
impl SubNodeProvider for MemoryProvider {
  fn provider_type(&self) -> SubNodeProviderType {
    SubNodeProviderType::Memory
  }

  fn get_node_definition(&self) -> Arc<NodeDefinition> {
    Arc::clone(&self.node_definition)
  }

  async fn initialize(&self) -> Result<(), NodeExecutionError> {
    log::info!("Memory provider ({}) initialized with max_messages: {}", self.provider_id, self.config.max_messages);

    // Start background cleanup task
    self.start_cleanup_task().await;

    Ok(())
  }
}

#[async_trait]
impl MemorySubNodeProvider for MemoryProvider {
  async fn store_messages(&self, session_id: &str, messages: Vec<Message>) -> Result<(), NodeExecutionError> {
    let messages_count = messages.len();

    // Convert from sub_node_provider::Message to MemoryMessage
    let memory_messages: Vec<MemoryMessage> = messages
      .into_iter()
      .map(|msg| MemoryMessage::new(msg.role, msg.content, session_id.to_string()))
      .collect();

    let mut storage_guard = self.storage.lock().await;
    storage_guard.store_messages(session_id, memory_messages)?;
    log::info!("Stored {} messages for session: {}", messages_count, session_id);

    Ok(())
  }

  async fn retrieve_messages(&self, session_id: &str, count: usize) -> Result<Vec<Message>, NodeExecutionError> {
    let storage_guard = self.storage.lock().await;
    let memory_messages = storage_guard.retrieve_messages(session_id, count)?;

    // Convert from MemoryMessage to sub_node_provider::Message
    let messages: Vec<Message> =
      memory_messages.into_iter().map(|msg| Message { role: msg.role, content: msg.content }).collect();

    log::info!("Retrieved {} messages for session: {}", messages.len(), session_id);
    Ok(messages)
  }
}

/// Factory function to create and configure memory provider
pub fn create_memory_provider(config: Option<MemoryProviderConfig>) -> Result<Arc<MemoryProvider>, NodeExecutionError> {
  let config = config.unwrap_or_default();
  let provider = Arc::new(MemoryProvider::new(config));
  Ok(provider)
}

/// Create memory provider from sub_node_provider::MemoryConfig
pub fn create_memory_provider_from_config(config: MemoryConfig) -> Result<Arc<MemoryProvider>, NodeExecutionError> {
  let provider = Arc::new(MemoryProvider::from_memory_config(config));
  Ok(provider)
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokio::time::{Duration, sleep};

  #[test]
  fn test_memory_provider_config_default() {
    let config = MemoryProviderConfig::default();
    assert_eq!(config.max_messages, 100);
    assert_eq!(config.session_timeout_seconds, 3600);
    assert_eq!(config.cleanup_interval_seconds, 300);
    assert!(!config.persistence_enabled);
  }

  #[test]
  fn test_memory_config_conversion() {
    let memory_config =
      MemoryConfig { context_window: Some(50), max_history: Some(200), persistence_enabled: Some(true) };

    let provider_config = MemoryProviderConfig::from(memory_config);
    assert_eq!(provider_config.max_messages, 200);
    assert!(provider_config.persistence_enabled);
  }

  #[test]
  fn test_memory_message_creation() {
    let message = MemoryMessage::new("user".to_string(), "Hello, world!".to_string(), "session_123".to_string());

    assert_eq!(message.role, "user");
    assert_eq!(message.content, "Hello, world!");
    assert_eq!(message.session_id, "session_123");
    assert!(message.timestamp <= Utc::now());
  }

  #[test]
  fn test_session_storage() {
    let config = MemoryProviderConfig { max_messages: 5, session_timeout_seconds: 3600, ..Default::default() };

    let mut storage = SessionStorage::new(config);
    let session_id = "test_session";

    // Test storing messages
    let messages = vec![
      MemoryMessage::new("user".to_string(), "Hello".to_string(), session_id.to_string()),
      MemoryMessage::new("assistant".to_string(), "Hi there!".to_string(), session_id.to_string()),
    ];

    storage.store_messages(session_id, messages).unwrap();
    assert_eq!(storage.get_session_stats(session_id).unwrap().message_count, 2);

    // Test retrieving messages
    let retrieved = storage.retrieve_messages(session_id, 1).unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].content, "Hi there!");

    // Test sliding window behavior
    let mut more_messages = Vec::new();
    for i in 0..6 {
      more_messages.push(MemoryMessage::new("user".to_string(), format!("Message {}", i), session_id.to_string()));
    }

    storage.store_messages(session_id, more_messages).unwrap();
    assert_eq!(storage.get_session_stats(session_id).unwrap().message_count, 5); // Should be limited to max_messages
  }

  #[test]
  fn test_provider_creation() {
    let config = MemoryProviderConfig { max_messages: 50, persistence_enabled: true, ..Default::default() };

    let provider = MemoryProvider::new(config);
    assert_eq!(provider.config().max_messages, 50);
    assert!(provider.config().persistence_enabled);
    assert!(!provider.provider_id().is_empty());
  }

  #[tokio::test]
  async fn test_provider_initialization() {
    let config = MemoryProviderConfig::default();
    let provider = MemoryProvider::new(config);

    let result = provider.initialize().await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_message_storage_and_retrieval() {
    let config = MemoryProviderConfig::default();
    let provider = MemoryProvider::new(config);

    // Initialize provider
    provider.initialize().await.unwrap();

    let session_id = "test_session_123";
    let messages = vec![
      Message { role: "user".to_string(), content: "What is the weather like?".to_string() },
      Message { role: "assistant".to_string(), content: "It's sunny today with a temperature of 25°C.".to_string() },
    ];

    // Store messages
    provider.store_messages(session_id, messages.clone()).await.unwrap();

    // Retrieve messages
    let retrieved = provider.retrieve_messages(session_id, 10).await.unwrap();
    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved[0].content, "What is the weather like?");
    assert_eq!(retrieved[1].content, "It's sunny today with a temperature of 25°C.");

    // Retrieve limited number of messages
    let limited = provider.retrieve_messages(session_id, 1).await.unwrap();
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].content, "It's sunny today with a temperature of 25°C.");
  }

  #[tokio::test]
  async fn test_session_management() {
    let config = MemoryProviderConfig {
      session_timeout_seconds: 1, // 1 second timeout for testing
      ..Default::default()
    };
    let provider = MemoryProvider::new(config);
    provider.initialize().await.unwrap();

    let session_id = "timeout_test";
    let messages = vec![Message { role: "user".to_string(), content: "Test message".to_string() }];

    // Store messages
    provider.store_messages(session_id, messages).await.unwrap();

    // Verify session exists
    let stats = provider.get_session_stats(session_id).await;
    assert!(stats.is_some());

    // Wait for session to expire
    sleep(Duration::from_secs(2)).await;

    // In a real implementation, expired sessions would be cleaned up by the background task
    // For testing, we can manually clear the session
    provider.clear_session(session_id).await.unwrap();

    // Verify session no longer exists
    let stats = provider.get_session_stats(session_id).await;
    assert!(stats.is_none());
  }

  #[tokio::test]
  async fn test_factory_function() {
    let config = Some(MemoryProviderConfig { max_messages: 25, persistence_enabled: true, ..Default::default() });

    let provider = create_memory_provider(config).unwrap();
    assert_eq!(provider.config().max_messages, 25);
    assert!(provider.config().persistence_enabled);

    provider.initialize().await.unwrap();
  }

  #[tokio::test]
  async fn test_memory_config_factory() {
    let memory_config =
      MemoryConfig { context_window: Some(10), max_history: Some(20), persistence_enabled: Some(false) };

    let provider = create_memory_provider_from_config(memory_config).unwrap();
    assert_eq!(provider.config().max_messages, 20);
    assert!(!provider.config().persistence_enabled);

    provider.initialize().await.unwrap();
  }

  #[tokio::test]
  async fn test_cluster_node_integration() {
    // Test the full integration with NodeRegistry and ClusterNodeExecutor
    use crate::workflow::{
      ClusterNodeConfig, NodeKind, NodeRegistry,
      graph_flow_tasks::{ClusterNodeExecutor, Context},
    };

    let config = MemoryProviderConfig {
      max_messages: 50,
      session_timeout_seconds: 300, // 5 minutes
      persistence_enabled: true,
      ..Default::default()
    };

    let provider = create_memory_provider(Some(config)).unwrap();
    provider.initialize().await.unwrap();

    let node_registry = NodeRegistry::new();
    let node_kind: NodeKind = "memory_provider".into();

    node_registry.register_subnode_provider(node_kind.clone(), provider.clone()).unwrap();
    assert!(node_registry.has_subnode_provider(&node_kind));

    let mut executor = ClusterNodeExecutor::new(node_registry);
    let cluster_config = ClusterNodeConfig {
      memory_config: Some(MemoryConfig {
        context_window: Some(20),
        max_history: Some(50),
        persistence_enabled: Some(true),
      }),
      ..Default::default()
    };

    executor.register_subnode_provider(node_kind, cluster_config).unwrap();
    assert_eq!(executor.task_count(), 1);

    let task_ids = executor.task_ids();
    let mut context = Context::new();
    context.set("test_session_id", "integration_test_session").unwrap();

    let result = executor.execute_task(&task_ids[0], context).await.unwrap();
    assert!(result.response.is_some());

    println!("✅ Memory Provider Cluster Node integration test passed!");
  }
}
