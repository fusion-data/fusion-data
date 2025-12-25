//! Agent configuration types
//!
//! This module provides agent configuration types. For the recommended
//! rig 0.27+ pattern, use [`factory::AgentConfig`] instead.
//!
//! # Migration from old API
//!
//! Old code using `ClientBuilderFactory`:
//!
//! ```ignore
//! let factory = ClientBuilderFactory::new();
//! let agent = factory.agent(&config)?;
//! ```
//!
//! New code using `ClientFactory`:
//!
//! ```ignore
//! let factory = ClientFactory::new();
//! let agent = factory.agent(&config)?;
//! ```

pub use super::client::AgentConfig;
