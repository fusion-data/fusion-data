//! Independent LLM provider nodes
//!
//! Each LLM provider is implemented as a separate node for better modularity
//! and reuse of common parameters and functionality.

use std::sync::Arc;

use hetumind_core::workflow::{NodeRegistry, RegistrationError};

pub mod lm_deepseek;
pub mod lm_moonshot;
pub mod lm_openai;
pub mod shared;

// Re-export all node implementations for easy access
pub use lm_deepseek::*;
pub use lm_moonshot::*;
pub use lm_openai::*;
pub use shared::*;

/// Register all LLM provider nodes with the given registry
pub fn register_nodes(node_registry: &NodeRegistry) -> Result<(), RegistrationError> {
  let deepseek_node = Arc::new(DeepseekModelNode::new()?);
  node_registry.register_node(deepseek_node)?;

  // 注册 Moonshot 节点
  let moonshot_node = Arc::new(MoonshotModelNode::new()?);
  node_registry.register_node(moonshot_node)?;

  // 注册 OpenAI 节点
  let openai_node = Arc::new(OpenaiModelNode::new()?);
  node_registry.register_node(openai_node)?;

  // TODO: Add other LLM provider nodes when rig-core integration is complete
  // - OpenAI node
  // - Anthropic node
  // - Google Gemini node
  // - XAI node
  // - Moonshot node
  // - Ollama node

  Ok(())
}
