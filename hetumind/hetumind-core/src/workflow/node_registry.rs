use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use dashmap::DashMap;

use crate::version::Version;
use crate::workflow::{FlowNodeRef, Node, NodeDescription, NodeType, RegistrationError, SubNodeRef};
use crate::workflow::{LLMSubNodeProviderRef, MemorySubNodeProviderRef, SubNodeType, ToolSubNodeProviderRef};

pub type NodeRef = Arc<dyn Node + Send + Sync>;

#[derive(Default)]
pub struct InnerNodeRegistry {
  nodes: DashMap<NodeType, NodeRef>,
  subnode_providers: DashMap<NodeType, SubNodeRef>,
  // Typed providers for safer and faster retrieval
  llm_suppliers: DashMap<NodeType, LLMSubNodeProviderRef>,
  memory_suppliers: DashMap<NodeType, MemorySubNodeProviderRef>,
  tool_suppliers: DashMap<NodeType, ToolSubNodeProviderRef>,
}

impl InnerNodeRegistry {
  /// Register a node executor
  ///
  /// Args:
  /// - `executable` - An executable node that implements the [Node] trait
  pub fn register_node(&self, executable: NodeRef) -> Result<(), RegistrationError> {
    let node_type = executable.node_type().clone();

    if self.contains(&node_type) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_type });
    }

    self.nodes.insert(node_type, executable);
    Ok(())
  }

  /// Get the default version of node executor for the given node kind
  ///
  /// Args:
  /// - `node_type` - The kind of the node to get the executor for
  ///
  /// Returns:
  /// - `Option<NodeExecutor>` - The node executor if found, otherwise None
  pub fn get_executor(&self, node_type: &NodeType) -> Option<FlowNodeRef> {
    self.nodes.get(node_type).and_then(|x| x.value().default_node_executor())
  }

  /// Get the node executor for the given node kind and version
  ///
  /// Args:
  /// - `node_type` - The kind of the node to get the executor for
  /// - `version` - The version of the node executor to get
  ///
  /// Returns:
  /// - `Option<NodeExecutor>` - The node executor if found, otherwise None
  pub fn get_executor_by_version(&self, node_type: &NodeType, version: &Version) -> Option<FlowNodeRef> {
    self.nodes.get(node_type).and_then(|x| x.value().get_node_executor(version))
  }

  /// Get the node supplier for the given node kind
  ///
  /// Args:
  /// - `node_type` - The kind of the node to get the supplier for
  ///
  /// Returns:
  /// - `Option<NodeSupplier>` - The node supplier if found, otherwise None
  pub fn get_supplier(&self, node_type: &NodeType) -> Option<SubNodeRef> {
    self.nodes.get(node_type).and_then(|x| x.value().default_node_supplier())
  }

  /// Get the node supplier for the given node kind and version
  ///
  /// Args:
  /// - `node_type` - The kind of the node to get the supplier for
  /// - `version` - The version of the node supplier to get
  ///
  /// Returns:
  /// - `Option<NodeSupplier>` - The node supplier if found, otherwise None
  pub fn get_supplier_by_version(&self, node_type: &NodeType, version: &Version) -> Option<SubNodeRef> {
    self.nodes.get(node_type).and_then(|x| x.value().get_node_supplier(version))
  }

  /// Get the default version of node definition for the given node kind
  ///
  /// Args:
  /// - `node_type` - The kind of the node to get the definition for
  ///
  /// Returns:
  /// - `Option<Arc<NodeDefinition>>` - The node definition if found, otherwise None
  pub fn get_definition(&self, node_type: &NodeType) -> Option<Arc<NodeDescription>> {
    self.get_executor(node_type).map(|node| node.description())
  }

  /// Get the node definition for the given node kind and version
  ///
  /// Args:
  /// - `node_type` - The kind of the node to get the definition for
  /// - `version` - The version of the node definition to get
  ///
  /// Returns:
  /// - `Option<Arc<NodeDefinition>>` - The node definition if found, otherwise None
  pub fn get_definition_by_version(&self, node_type: &NodeType, version: &Version) -> Option<Arc<NodeDescription>> {
    self.get_executor_by_version(node_type, version).map(|node| node.description())
  }

  /// Check if a node kind is registered
  pub fn contains(&self, node_type: &NodeType) -> bool {
    self.nodes.contains_key(node_type)
  }

  /// Get all registered node kinds
  pub fn registered_node_kinds(&self) -> Vec<NodeType> {
    self.nodes.iter().map(|x| x.key().clone()).collect()
  }

  /// Unregister a node executor
  ///
  /// Args:
  /// - `node_type` - The kind of the node to unregister
  ///
  /// Returns:
  /// - `Option<NodeRef>` - The unregistered node executor if found, otherwise None
  pub fn unregister_node(&self, node_type: &NodeType) -> Option<NodeRef> {
    self.nodes.remove(node_type).map(|(_, node)| node)
  }

  /// Clear all registered node executors
  pub fn clear(&self) {
    self.nodes.clear();
  }

  /// Get the number of registered node executors
  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  /// Check if the registry is empty
  ///
  /// Returns:
  /// - `bool` - True if the registry is empty, otherwise false
  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  // ===== SubNodeProvider Management Methods =====

  /// Register a SubNodeProvider for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to register the provider for
  /// - `provider` - The SubNodeProvider to register
  ///
  /// Returns:
  /// - `Result<(), RegistrationError>` - Ok if successful, Err if provider already exists
  pub fn register_subnode_provider(&self, kind: NodeType, provider: SubNodeRef) -> Result<(), RegistrationError> {
    if self.subnode_providers.contains_key(&kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_type: kind });
    }

    self.subnode_providers.insert(kind, provider);
    Ok(())
  }

  // ===== Typed SubNodeProvider Registration =====

  /// Register a LLMSubNodeProvider for a given node kind (typed)
  pub fn register_llm_supplier(
    &self,
    kind: NodeType,
    provider: LLMSubNodeProviderRef,
  ) -> Result<(), RegistrationError> {
    if self.llm_suppliers.contains_key(&kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_type: kind });
    }
    self.llm_suppliers.insert(kind, provider);
    Ok(())
  }

  /// Register a MemorySubNodeProvider for a given node kind (typed)
  pub fn register_memory_supplier(
    &self,
    kind: NodeType,
    provider: MemorySubNodeProviderRef,
  ) -> Result<(), RegistrationError> {
    if self.memory_suppliers.contains_key(&kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_type: kind });
    }
    self.memory_suppliers.insert(kind, provider);
    Ok(())
  }

  /// Register a ToolSubNodeProvider for a given node kind (typed)
  pub fn register_tool_supplier(
    &self,
    kind: NodeType,
    provider: ToolSubNodeProviderRef,
  ) -> Result<(), RegistrationError> {
    if self.tool_suppliers.contains_key(&kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_type: kind });
    }
    self.tool_suppliers.insert(kind, provider);
    Ok(())
  }

  /// Get the SubNodeProvider for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to get the provider for
  ///
  /// Returns:
  /// - `Option<SubNodeProviderRef>` - The provider if found, otherwise None
  pub fn get_subnode_provider(&self, kind: &NodeType) -> Option<SubNodeRef> {
    self.subnode_providers.get(kind).map(|provider| provider.clone())
  }

  /// Unregister a SubNodeProvider for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to unregister the provider for
  ///
  /// Returns:
  /// - `Option<SubNodeProviderRef>` - The unregistered provider if found, otherwise None
  pub fn unregister_subnode_provider(&self, kind: &NodeType) -> Option<SubNodeRef> {
    self.subnode_providers.remove(kind).map(|(_, provider)| provider)
  }

  /// Get all registered SubNodeProvider kinds
  ///
  /// Returns:
  /// - `Vec<NodeKind>` - Vector of all registered node kinds with SubNodeProviders
  pub fn registered_subnode_provider_kinds(&self) -> Vec<NodeType> {
    self.subnode_providers.iter().map(|entry| entry.key().clone()).collect()
  }

  /// Check if a SubNodeProvider is registered for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to check
  ///
  /// Returns:
  /// - `bool` - True if a provider is registered, otherwise false
  pub fn has_subnode_provider(&self, kind: &NodeType) -> bool {
    self.subnode_providers.contains_key(kind)
  }

  /// Get the number of registered SubNodeProviders
  ///
  /// Returns:
  /// - `usize` - The count of registered SubNodeProviders
  pub fn subnode_provider_count(&self) -> usize {
    self.subnode_providers.len()
  }

  /// Clear all registered SubNodeProviders
  pub fn clear_subnode_providers(&self) {
    self.subnode_providers.clear();
  }

  // ===== Typed SubNodeProvider Getters (预备，后续支持 downcast) =====

  /// 获取 LLM 类型的 SubNodeProvider（当前返回 SubNodeRef，后续将支持 typed 引用）
  pub fn get_llm_supplier(&self, kind: &NodeType) -> Option<SubNodeRef> {
    self
      .subnode_providers
      .get(kind)
      .filter(|p| p.provider_type() == SubNodeType::LLM)
      .map(|p| p.clone())
  }

  /// 获取 LLM 类型的 SubNodeProvider（typed 引用）
  pub fn get_llm_supplier_typed(&self, kind: &NodeType) -> Option<LLMSubNodeProviderRef> {
    self.llm_suppliers.get(kind).map(|p| p.clone())
  }

  /// 获取 Memory 类型的 SubNodeProvider（当前返回 SubNodeRef，后续将支持 typed 引用）
  pub fn get_memory_supplier(&self, kind: &NodeType) -> Option<SubNodeRef> {
    self
      .subnode_providers
      .get(kind)
      .filter(|p| p.provider_type() == SubNodeType::Memory)
      .map(|p| p.clone())
  }

  /// 获取 Memory 类型的 SubNodeProvider（typed 引用）
  pub fn get_memory_supplier_typed(&self, kind: &NodeType) -> Option<MemorySubNodeProviderRef> {
    self.memory_suppliers.get(kind).map(|p| p.clone())
  }

  /// 获取 Tool 类型的 SubNodeProvider（当前返回 SubNodeRef，后续将支持 typed 引用）
  pub fn get_tool_supplier(&self, kind: &NodeType) -> Option<SubNodeRef> {
    self
      .subnode_providers
      .get(kind)
      .filter(|p| p.provider_type() == SubNodeType::Tool)
      .map(|p| p.clone())
  }

  /// 获取 Tool 类型的 SubNodeProvider（typed 引用）
  pub fn get_tool_supplier_typed(&self, kind: &NodeType) -> Option<ToolSubNodeProviderRef> {
    self.tool_suppliers.get(kind).map(|p| p.clone())
  }

  /// 获取 Agent 类型的 SubNodeProvider（当前返回 SubNodeRef，后续将支持 typed 引用）
  pub fn get_agent_supplier(&self, kind: &NodeType) -> Option<SubNodeRef> {
    self
      .subnode_providers
      .get(kind)
      .filter(|p| p.provider_type() == SubNodeType::Agent)
      .map(|p| p.clone())
  }
}

/// Node registry for managing node executors and related metadata
#[derive(Clone, Default)]
pub struct NodeRegistry(Arc<InnerNodeRegistry>);

impl Deref for NodeRegistry {
  type Target = Arc<InnerNodeRegistry>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for NodeRegistry {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl NodeRegistry {
  pub fn new() -> Self {
    Self::default()
  }
}
