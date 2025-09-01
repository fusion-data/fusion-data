pub use axum::Router;

pub mod config;
mod error;
pub mod extract;
pub mod middleware;
pub mod server;
mod util;

pub use error::{WebError, WebResult};
pub use util::*;
