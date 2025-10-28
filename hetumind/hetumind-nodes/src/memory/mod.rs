//! Memory module for Simple Memory Node and related memory management functionality

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

pub mod graph_flow_memory;
pub mod simple_memory_node;

// Re-export for easy access
pub use graph_flow_memory::*;
pub use simple_memory_node::*;

/// Register all memory nodes with the given registry
pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  simple_memory_node::register_nodes(node_registry)?;

  // TODO: Add other memory nodes when needed
  // - External database memory nodes (Redis, PostgreSQL, etc.)
  // - File-based memory nodes
  // - Vector memory nodes for semantic search

  Ok(())
}
