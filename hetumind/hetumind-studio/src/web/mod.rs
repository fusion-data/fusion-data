//! Web 模块
pub mod remote_authz_middleware;

// 示例模块 - 展示如何使用权限系统
#[cfg(feature = "examples")]
pub mod workflow_example;

#[cfg(feature = "examples")]
pub mod authz_example;

pub use remote_authz_middleware::*;