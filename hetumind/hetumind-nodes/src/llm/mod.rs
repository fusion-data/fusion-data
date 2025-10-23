//! Independent LLM provider nodes
//!
//! Each LLM provider is implemented as a separate node for better modularity
//! and reuse of common parameters and functionality.

use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

pub mod deepseek_node;
pub mod shared;

// Re-export all node implementations for easy access
pub use deepseek_node::*;
pub use shared::*;

/// Register all LLM provider nodes with the given registry
pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let deepseek_node = Arc::new(DeepseekModelNode::new()?);
  node_registry.register_node(deepseek_node)?;

  // TODO: Add other LLM provider nodes when rig-core integration is complete
  // - OpenAI node
  // - Anthropic node
  // - Google Gemini node
  // - XAI node
  // - Moonshot node
  // - Ollama node

  Ok(())
}
