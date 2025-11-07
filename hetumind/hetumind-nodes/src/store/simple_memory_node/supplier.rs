use std::sync::Arc;

use async_trait::async_trait;
use fusion_core::application::Application;
use hetumind_context::services::memory_service::MemoryService;
use hetumind_core::version::Version;
use hetumind_core::workflow::ExecutionContext;
use hetumind_core::workflow::{
  MemorySubNodeProvider, Message, NodeDescription, NodeExecutionError, NodeGroupKind, SubNode, SubNodeType,
};

use crate::store::simple_memory_node::SIMPLE_MEMORY_NODE_KIND;

/// Simple Memory Supplier 提供 MemorySubNodeProvider 能力，基于独立 Memory Service 存储/检索消息
pub struct SimpleMemorySupplier {
  definition: Arc<NodeDescription>,
}

impl Default for SimpleMemorySupplier {
  fn default() -> Self {
    Self::new()
  }
}

impl SimpleMemorySupplier {
  pub fn new() -> Self {
    Self { definition: Arc::new(Self::create_definition()) }
  }

  /// 创建 Supplier 的节点定义
  fn create_definition() -> NodeDescription {
    NodeDescription::new(SIMPLE_MEMORY_NODE_KIND, "Simple Memory Supplier")
      .with_version(Version::new(1, 0, 0))
      .add_group(NodeGroupKind::Transform)
      .with_description("Provide memory store/retrieve via Memory Service component")
      .add_output(hetumind_core::workflow::OutputPortConfig::new(
        hetumind_core::workflow::NodeConnectionKind::AiMemory,
        "Memory",
      ))
  }
}

#[async_trait]
impl SubNode for SimpleMemorySupplier {
  /// Provider 类型：Memory
  fn provider_type(&self) -> SubNodeType {
    SubNodeType::Memory
  }

  /// Supplier 的节点定义
  fn description(&self) -> Arc<NodeDescription> {
    self.definition.clone()
  }

  /// 初始化 Supplier（当前无状态，直接返回 Ok）
  async fn initialize(&self) -> Result<(), NodeExecutionError> {
    Ok(())
  }

  /// 返回 Any 引用用于安全 downcast（typed 获取）
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}

#[async_trait]
impl MemorySubNodeProvider for SimpleMemorySupplier {
  /// 存储消息到指定会话（通过 Memory Service）
  async fn store_messages(&self, session_id: &str, messages: Vec<Message>) -> Result<(), NodeExecutionError> {
    let svc = Application::global()
      .get_component::<Arc<dyn MemoryService>>()
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("MemoryService component not found: {}", e)))?;

    // TODO: 从上下文注入租户/工作流信息；当前占位使用默认租户与工作流
    let tenant_id = "default_tenant";
    let workflow_id = "default_workflow";
    svc.store_messages(tenant_id, workflow_id, session_id, messages)
  }

  /// 从指定会话检索最近 N 条消息（通过 Memory Service）
  async fn retrieve_messages(&self, session_id: &str, count: usize) -> Result<Vec<Message>, NodeExecutionError> {
    let svc = Application::global()
      .get_component::<Arc<dyn MemoryService>>()
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("MemoryService component not found: {}", e)))?;
    let tenant_id = "default_tenant";
    let workflow_id = "default_workflow";
    svc.retrieve_messages(tenant_id, workflow_id, session_id, count)
  }
}

impl SimpleMemorySupplier {
  /// 使用 ExecutionContext 注入租户/工作流信息，存储消息到指定会话
  pub async fn store_messages_with_ctx(
    &self,
    exec_ctx: &ExecutionContext,
    session_id: &str,
    messages: Vec<Message>,
  ) -> Result<(), NodeExecutionError> {
    let svc = Application::global()
      .get_component::<Arc<dyn MemoryService>>()
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("MemoryService component not found: {}", e)))?;
    let tenant_id = exec_ctx.ctx().tenant_id().to_string();
    let workflow_id = exec_ctx.workflow().id.to_string();
    svc.store_messages(&tenant_id, &workflow_id, session_id, messages)
  }

  /// 使用 ExecutionContext 注入租户/工作流信息，检索指定会话的最近 N 条消息
  pub async fn retrieve_messages_with_ctx(
    &self,
    exec_ctx: &ExecutionContext,
    session_id: &str,
    count: usize,
  ) -> Result<Vec<Message>, NodeExecutionError> {
    let svc = Application::global()
      .get_component::<Arc<dyn MemoryService>>()
      .map_err(|e| NodeExecutionError::ConfigurationError(format!("MemoryService component not found: {}", e)))?;
    let tenant_id = exec_ctx.ctx().tenant_id().to_string();
    let workflow_id = exec_ctx.workflow().id.to_string();
    svc.retrieve_messages(&tenant_id, &workflow_id, session_id, count)
  }
}
