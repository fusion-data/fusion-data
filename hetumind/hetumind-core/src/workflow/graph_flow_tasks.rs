//! GraphFlow Task implementations for Cluster Node architecture
//!
//! This module provides simplified GraphFlow Task implementations that integrate with SubNodeProviders
//! to enable flexible workflow execution within the Cluster Node paradigm.
use std::sync::Arc;

use async_trait::async_trait;
use fusion_common::ahash;

use crate::workflow::{
  NodeExecutionError, NodeKind, NodeRegistry,
  sub_node::{
    AgentConfig, AgentSubNodeProviderRef, ClusterNodeConfig, ExecutionConfig, LLMConfig, LLMSubNodeProviderRef,
    MemoryConfig, MemorySubNodeProviderRef, Message, SubNodeRef, SubNodeType, ToolSubNodeProviderRef,
  },
};

// Simplified GraphFlow types for now
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextAction {
  Continue,
  End,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
  pub response: Option<String>,
  pub next_action: NextAction,
}

impl TaskResult {
  pub fn new(response: Option<String>, next_action: NextAction) -> Self {
    Self { response, next_action }
  }
}

#[async_trait]
pub trait Task: Send + Sync {
  async fn run(&self, context: Context) -> Result<TaskResult, NodeExecutionError>;
  fn id(&self) -> &str;
}

/// Simplified context for GraphFlow execution
#[derive(Debug, Clone, Default)]
pub struct Context {
  data: ahash::HashMap<String, serde_json::Value>,
}

impl Context {
  pub fn new() -> Self {
    Self { data: ahash::HashMap::default() }
  }

  pub fn get<V: serde::de::DeserializeOwned>(&self, key: &str) -> Option<V> {
    self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
  }

  pub fn set<V: serde::Serialize>(&mut self, key: &str, value: V) -> Result<(), NodeExecutionError> {
    self.data.insert(
      key.to_string(),
      serde_json::to_value(value).map_err(|e| NodeExecutionError::DataProcessingError {
        message: format!("Failed to serialize value: {}", e),
      })?,
    );
    Ok(())
  }
}

/// Type alias for convenience
pub type TaskRef = Arc<dyn Task + Send + Sync>;

/// Base GraphFlow task for SubNodeProvider execution
pub struct SubNodeProviderTask {
  provider: SubNodeRef,
  node_kind: NodeKind,
  execution_config: ExecutionConfig,
  task_id: String,
}

impl SubNodeProviderTask {
  /// Create a new SubNodeProvider task
  pub fn new(provider: SubNodeRef, node_kind: NodeKind, execution_config: ExecutionConfig) -> Self {
    let task_id = node_kind.to_string();
    Self { provider, node_kind, execution_config, task_id }
  }

  /// Get the provider type
  fn provider_type(&self) -> SubNodeType {
    self.provider.provider_type()
  }
}

#[async_trait]
impl Task for SubNodeProviderTask {
  async fn run(&self, mut context: Context) -> Result<TaskResult, NodeExecutionError> {
    // Initialize the provider if needed
    self.provider.initialize().await?;

    // Execute based on provider type
    match self.provider_type() {
      SubNodeType::LLM => {
        // This will be handled by specialized LLM tasks
        Ok(TaskResult::new(Some("LLM execution handled by LLMProviderTask".to_string()), NextAction::Continue))
      }
      SubNodeType::Memory => {
        // This will be handled by specialized Memory tasks
        Ok(TaskResult::new(Some("Memory execution handled by MemoryProviderTask".to_string()), NextAction::Continue))
      }
      SubNodeType::Tool => {
        // This will be handled by specialized Tool tasks
        Ok(TaskResult::new(Some("Tool execution handled by ToolProviderTask".to_string()), NextAction::Continue))
      }
      SubNodeType::Agent => {
        // This will be handled by specialized Agent tasks
        Ok(TaskResult::new(Some("Agent execution handled by AgentProviderTask".to_string()), NextAction::Continue))
      }
    }
  }

  fn id(&self) -> &str {
    &self.task_id
  }
}

/// LLM Provider GraphFlow Task
pub struct LLMProviderTask {
  llm_provider: LLMSubNodeProviderRef,
  node_kind: NodeKind,
  default_config: LLMConfig,
  task_id: String,
}

impl LLMProviderTask {
  /// Create a new LLM provider task
  pub fn new(llm_provider: LLMSubNodeProviderRef, node_kind: NodeKind, default_config: LLMConfig) -> Self {
    let task_id = format!("llm_{}", node_kind);
    Self { llm_provider, node_kind, default_config, task_id }
  }
}

#[async_trait]
impl Task for LLMProviderTask {
  async fn run(&self, mut _context: Context) -> Result<TaskResult, NodeExecutionError> {
    // Initialize the provider
    self.llm_provider.initialize().await?;

    // For now, return a placeholder result
    // In a full implementation, this would call the LLM provider
    let result = format!("LLM {} would execute with config: {:?}", self.node_kind, self.default_config);

    Ok(TaskResult::new(Some(result), NextAction::Continue))
  }

  fn id(&self) -> &str {
    &self.task_id
  }
}

/// Memory Provider GraphFlow Task
pub struct MemoryProviderTask {
  memory_provider: MemorySubNodeProviderRef,
  node_kind: NodeKind,
  default_config: MemoryConfig,
  task_id: String,
}

impl MemoryProviderTask {
  /// Create a new memory provider task
  pub fn new(memory_provider: MemorySubNodeProviderRef, node_kind: NodeKind, default_config: MemoryConfig) -> Self {
    let task_id = format!("memory_{}", node_kind);
    Self { memory_provider, node_kind, default_config, task_id }
  }
}

#[async_trait]
impl Task for MemoryProviderTask {
  async fn run(&self, mut _context: Context) -> Result<TaskResult, NodeExecutionError> {
    // Initialize the provider
    self.memory_provider.initialize().await?;

    // For now, return a placeholder result
    // In a full implementation, this would call the Memory provider
    let result = format!("Memory {} would execute with config: {:?}", self.node_kind, self.default_config);

    Ok(TaskResult::new(Some(result), NextAction::Continue))
  }

  fn id(&self) -> &str {
    &self.task_id
  }
}

/// Tool Provider GraphFlow Task
pub struct ToolProviderTask {
  tool_provider: ToolSubNodeProviderRef,
  node_kind: NodeKind,
  task_id: String,
}

impl ToolProviderTask {
  /// Create a new tool provider task
  pub fn new(tool_provider: ToolSubNodeProviderRef, node_kind: NodeKind) -> Self {
    let task_id = format!("tool_{}", node_kind);
    Self { tool_provider, node_kind, task_id }
  }
}

#[async_trait]
impl Task for ToolProviderTask {
  async fn run(&self, mut context: Context) -> Result<TaskResult, NodeExecutionError> {
    // Initialize the provider
    self.tool_provider.initialize().await?;

    // For now, return a placeholder result
    // In a full implementation, this would call the Tool provider
    let result = format!("Tool {} would execute", self.node_kind);

    Ok(TaskResult::new(Some(result), NextAction::Continue))
  }

  fn id(&self) -> &str {
    &self.task_id
  }
}

/// Agent Provider GraphFlow Task
pub struct AgentProviderTask {
  agent_provider: AgentSubNodeProviderRef,
  node_kind: NodeKind,
  default_config: AgentConfig,
  task_id: String,
}

impl AgentProviderTask {
  /// Create a new Agent provider task
  pub fn new(agent_provider: AgentSubNodeProviderRef, node_kind: NodeKind, default_config: AgentConfig) -> Self {
    let task_id = format!("agent_{}", node_kind);
    Self { agent_provider, node_kind, default_config, task_id }
  }
}

#[async_trait]
impl Task for AgentProviderTask {
  async fn run(&self, mut context: Context) -> Result<TaskResult, NodeExecutionError> {
    // Initialize the provider
    self.agent_provider.initialize().await?;

    // Extract input messages from context
    let input_messages = match context.get::<serde_json::Value>("input_messages") {
      Some(value) => {
        // Try to parse as JSON array of messages
        serde_json::from_value::<Vec<Message>>(value.clone()).unwrap_or_else(|_| {
          // Fallback: create a single message from the value
          vec![Message { role: "user".to_string(), content: value.to_string() }]
        })
      }
      None => {
        vec![Message { role: "user".to_string(), content: "Hello, Agent!".to_string() }]
      }
    };

    // Execute agent
    let response = self.agent_provider.execute_agent(input_messages, self.default_config.clone()).await?;

    // Store response in context for potential next steps
    context.set("agent_response", response.content.clone())?;

    Ok(TaskResult::new(Some(response.content), NextAction::Continue))
  }

  fn id(&self) -> &str {
    &self.task_id
  }
}

/// Cluster Node execution coordinator that manages multiple SubNodeProvider tasks
pub struct ClusterNodeExecutor {
  node_registry: Arc<NodeRegistry>,
  tasks: ahash::HashMap<String, TaskRef>,
}

impl std::fmt::Debug for ClusterNodeExecutor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ClusterNodeExecutor")
      .field("node_registry", &"NodeRegistry")
      .field("tasks_count", &self.tasks.len())
      .finish()
  }
}

impl ClusterNodeExecutor {
  /// Create a new cluster node executor
  pub fn new(node_registry: NodeRegistry) -> Self {
    Self { node_registry: Arc::new(node_registry), tasks: ahash::HashMap::default() }
  }

  /// Create a new cluster node executor with shared registry
  pub fn new_with_shared_registry(node_registry: Arc<NodeRegistry>) -> Self {
    Self { node_registry, tasks: ahash::HashMap::default() }
  }

  /// Create a new context for execution
  pub fn create_context(&self) -> Context {
    Context::new()
  }

  /// Register a SubNodeProvider and create corresponding GraphFlow tasks
  pub fn register_subnode_provider(
    &mut self,
    kind: NodeKind,
    config: ClusterNodeConfig,
  ) -> Result<(), NodeExecutionError> {
    // Get the SubNodeProvider from registry
    let provider = self.node_registry.get_subnode_provider(&kind).ok_or_else(|| {
      NodeExecutionError::ConfigurationError(format!("No SubNodeProvider registered for kind: {:?}", kind))
    })?;

    // Create appropriate task based on provider type
    let task: TaskRef = match provider.provider_type() {
      SubNodeType::LLM => {
        // For now, create a simple placeholder task
        // In a full implementation, this would downcast and create proper LLM tasks
        let default_config = config.llm_config.unwrap_or_default();
        Arc::new(SubNodeProviderTask::new(provider, kind.clone(), config.execution_config.clone()))
      }
      SubNodeType::Memory => {
        let default_config = config.memory_config.unwrap_or_default();
        Arc::new(SubNodeProviderTask::new(provider, kind.clone(), config.execution_config.clone()))
      }
      SubNodeType::Tool => Arc::new(SubNodeProviderTask::new(provider, kind.clone(), config.execution_config.clone())),
      SubNodeType::Agent => {
        let default_config = config.agent_config.unwrap_or_default();
        // For now, create a simple placeholder task
        // In a full implementation, this would downcast and create proper Agent tasks
        Arc::new(SubNodeProviderTask::new(provider, kind.clone(), config.execution_config.clone()))
      }
    };

    // Register the task
    self.tasks.insert(task.id().to_string(), task);
    Ok(())
  }

  /// Execute a task by ID
  pub async fn execute_task(&self, task_id: &str, mut context: Context) -> Result<TaskResult, NodeExecutionError> {
    let task = self
      .tasks
      .get(task_id)
      .ok_or_else(|| NodeExecutionError::ConfigurationError(format!("Task '{}' not found", task_id)))?;

    task.run(context).await
  }

  /// Get all registered task IDs
  pub fn task_ids(&self) -> Vec<String> {
    self.tasks.keys().cloned().collect()
  }

  /// Get the number of registered tasks
  pub fn task_count(&self) -> usize {
    self.tasks.len()
  }
}
