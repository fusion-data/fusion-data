use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use dashmap::DashMap;

use super::sub_node_provider::{SubNodeProviderRef, SubNodeProviderType};
use crate::workflow::{Node, NodeDefinition, NodeExecutor, NodeKind, RegistrationError};
use crate::{version::Version, workflow::NodeSupplier};

pub type NodeRef = Arc<dyn Node + Send + Sync>;

#[derive(Default)]
pub struct InnerNodeRegistry {
  nodes: DashMap<NodeKind, NodeRef>,
  subnode_providers: DashMap<NodeKind, SubNodeProviderRef>,
}

impl InnerNodeRegistry {
  /// Register a node executor
  ///
  /// Args:
  /// - `executable` - An executable node that implements the [Node] trait
  pub fn register_node(&self, executable: NodeRef) -> Result<(), RegistrationError> {
    let node_kind = executable.kind().clone();

    if self.contains(&node_kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_kind });
    }

    self.nodes.insert(node_kind, executable);
    Ok(())
  }

  /// Get the default version of node executor for the given node kind
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to get the executor for
  ///
  /// Returns:
  /// - `Option<NodeExecutor>` - The node executor if found, otherwise None
  pub fn get_executor(&self, node_kind: &NodeKind) -> Option<NodeExecutor> {
    self.nodes.get(node_kind).and_then(|x| x.value().default_node_executor())
  }

  /// Get the node executor for the given node kind and version
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to get the executor for
  /// - `version` - The version of the node executor to get
  ///
  /// Returns:
  /// - `Option<NodeExecutor>` - The node executor if found, otherwise None
  pub fn get_executor_by_version(&self, node_kind: &NodeKind, version: &Version) -> Option<NodeExecutor> {
    self.nodes.get(node_kind).and_then(|x| x.value().get_node_executor(version))
  }

  /// Get the node supplier for the given node kind
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to get the supplier for
  ///
  /// Returns:
  /// - `Option<NodeSupplier>` - The node supplier if found, otherwise None
  pub fn get_supplier(&self, node_kind: &NodeKind) -> Option<NodeSupplier> {
    self.nodes.get(node_kind).and_then(|x| x.value().default_node_supplier())
  }

  /// Get the node supplier for the given node kind and version
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to get the supplier for
  /// - `version` - The version of the node supplier to get
  ///
  /// Returns:
  /// - `Option<NodeSupplier>` - The node supplier if found, otherwise None
  pub fn get_supplier_by_version(&self, node_kind: &NodeKind, version: &Version) -> Option<NodeSupplier> {
    self.nodes.get(node_kind).and_then(|x| x.value().get_node_supplier(version))
  }

  /// Get the default version of node definition for the given node kind
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to get the definition for
  ///
  /// Returns:
  /// - `Option<Arc<NodeDefinition>>` - The node definition if found, otherwise None
  pub fn get_definition(&self, node_kind: &NodeKind) -> Option<Arc<NodeDefinition>> {
    self.get_executor(node_kind).map(|node| node.definition())
  }

  /// Get the node definition for the given node kind and version
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to get the definition for
  /// - `version` - The version of the node definition to get
  ///
  /// Returns:
  /// - `Option<Arc<NodeDefinition>>` - The node definition if found, otherwise None
  pub fn get_definition_by_version(&self, node_kind: &NodeKind, version: &Version) -> Option<Arc<NodeDefinition>> {
    self.get_executor_by_version(node_kind, version).map(|node| node.definition())
  }

  /// Check if a node kind is registered
  pub fn contains(&self, node_kind: &NodeKind) -> bool {
    self.nodes.contains_key(node_kind)
  }

  /// Get all registered node kinds
  pub fn registered_node_kinds(&self) -> Vec<NodeKind> {
    self.nodes.iter().map(|x| x.key().clone()).collect()
  }

  /// Unregister a node executor
  ///
  /// Args:
  /// - `node_kind` - The kind of the node to unregister
  ///
  /// Returns:
  /// - `Option<NodeRef>` - The unregistered node executor if found, otherwise None
  pub fn unregister_node(&self, node_kind: &NodeKind) -> Option<NodeRef> {
    self.nodes.remove(node_kind).map(|(_, node)| node)
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
  pub fn register_subnode_provider(
    &self,
    kind: NodeKind,
    provider: SubNodeProviderRef,
  ) -> Result<(), RegistrationError> {
    if self.subnode_providers.contains_key(&kind) {
      return Err(RegistrationError::NodeKindAlreadyExists { node_kind: kind });
    }

    self.subnode_providers.insert(kind, provider);
    Ok(())
  }

  /// Get the SubNodeProvider for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to get the provider for
  ///
  /// Returns:
  /// - `Option<SubNodeProviderRef>` - The provider if found, otherwise None
  pub fn get_subnode_provider(&self, kind: &NodeKind) -> Option<SubNodeProviderRef> {
    self.subnode_providers.get(kind).map(|provider| provider.clone())
  }

  /// Unregister a SubNodeProvider for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to unregister the provider for
  ///
  /// Returns:
  /// - `Option<SubNodeProviderRef>` - The unregistered provider if found, otherwise None
  pub fn unregister_subnode_provider(&self, kind: &NodeKind) -> Option<SubNodeProviderRef> {
    self.subnode_providers.remove(kind).map(|(_, provider)| provider)
  }

  /// Get all registered SubNodeProvider kinds
  ///
  /// Returns:
  /// - `Vec<NodeKind>` - Vector of all registered node kinds with SubNodeProviders
  pub fn registered_subnode_provider_kinds(&self) -> Vec<NodeKind> {
    self.subnode_providers.iter().map(|entry| entry.key().clone()).collect()
  }

  /// Check if a SubNodeProvider is registered for a given node kind
  ///
  /// Args:
  /// - `kind` - The node kind to check
  ///
  /// Returns:
  /// - `bool` - True if a provider is registered, otherwise false
  pub fn has_subnode_provider(&self, kind: &NodeKind) -> bool {
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
