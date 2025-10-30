pub mod agents;
pub mod client;
pub mod embeddings;
mod error;
pub mod graph_flow;
pub mod json_utils;
pub mod providers;
pub mod utils;

pub use error::*;
/// Re-export rig for easier access to core types
pub use rig;

pub struct DefaultProviders;
impl DefaultProviders {
  pub const ANTHROPIC: &'static str = "anthropic";
  pub const COHERE: &'static str = "cohere";
  pub const GEMINI: &'static str = "gemini";
  pub const HUGGINGFACE: &'static str = "huggingface";
  pub const OPENAI: &'static str = "openai";
  pub const OPENAI_COMPATIBLE: &'static str = "openai-compatible";
  pub const OPENROUTER: &'static str = "openrouter";
  pub const TOGETHER: &'static str = "together";
  pub const XAI: &'static str = "xai";
  pub const AZURE: &'static str = "azure";
  pub const DEEPSEEK: &'static str = "deepseek";
  pub const GALADRIEL: &'static str = "galadriel";
  pub const GROQ: &'static str = "groq";
  pub const HYPERBOLIC: &'static str = "hyperbolic";
  pub const MOONSHOT: &'static str = "moonshot";
  pub const MIRA: &'static str = "mira";
  pub const MISTRAL: &'static str = "mistral";
  pub const OLLAMA: &'static str = "ollama";
  pub const PERPLEXITY: &'static str = "perplexity";
}
