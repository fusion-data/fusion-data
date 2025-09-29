use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use dashmap::DashMap;

use crate::version::Version;
use crate::workflow::{Node, NodeDefinition, NodeExecutor, NodeKind, RegistrationError};

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

pub type NodeRef = Arc<dyn Node + Send + Sync>;

#[derive(Default)]
pub struct InnerNodeRegistry {
  nodes: DashMap<NodeKind, NodeRef>,
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

  // /// Get all registered default version of node definitions
  // pub fn all_definitions(&self) -> Vec<Arc<NodeDefinition>> {
  //   self
  //     .nodes
  //     .iter()
  //     .filter_map(|x| x.value().default_node_executor().map(|node| node.definition().clone()))
  //     .collect()
  // }

  // /// Get all registered default version of node definitions for the given node group kind
  // pub fn definitions_by_group(&self, group: &NodeGroupKind) -> Vec<Arc<NodeDefinition>> {
  //   self
  //     .nodes
  //     .iter()
  //     .filter_map(|x| x.value().default_node_executor().map(|node| node.definition()))
  //     .filter(|x| x.groups.contains(group))
  //     .collect()
  // }

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
}
