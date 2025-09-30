//! # Hetuflow SDK
//!
//! Rust SDK for Hetuflow distributed task scheduling and workflow orchestration system.
//!
//! ## Features
//!
//! - **Native Support**: Full support for native Rust applications with Tokio runtime
//! - **WASM Support**: Compile to WebAssembly for browser and Node.js environments
//! - **Type Safety**: Full type safety using models from hetuflow-core
//! - **Async/Await**: Async first design with both native and WASM support
//! - **Error Handling**: Comprehensive error handling with detailed error messages
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hetuflow_sdk::HetuflowClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = HetuflowClient::new("http://localhost:8080")?;
//!
//!     // List agents
//!     let agents = client.agents().query(Default::default()).await?;
//!     println!("Found {} agents", agents.total);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## WASM Support
//!
//! ```rust,no_run
//! use hetuflow_sdk::HetuflowClient;
//! use wasm_bindgen_futures::spawn_local;
//!
//! let client = HetuflowClient::new("http://localhost:8080")?;
//! spawn_local(async move {
//!     let agents = client.agents().query(Default::default()).await.unwrap();
//!     web_sys::console::log_1(&format!("Found {} agents", agents.total).into());
//! });
//! ```

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod client;
mod config;
mod error;
pub mod models;

pub use client::HetuflowClient;
pub use config::Config;
pub use error::{SdkError, SdkResult};

// Re-export models from hetuflow-core
pub use hetuflow_core::models::*;

// API modules
pub mod apis;

// Platform detection for conditional compilation
#[cfg(not(target_arch = "wasm32"))]
mod platform {
  pub use reqwest::Response;
}

#[cfg(target_arch = "wasm32")]
mod platform {
  pub use gloo_net::http::Response;
}
