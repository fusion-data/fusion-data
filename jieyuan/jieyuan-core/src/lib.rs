pub mod model;
pub mod utils;
pub mod web;

/// Common Error type for jieyuan-core
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
