//! hetus - Rust 数据融合平台聚合包
//!
//! # 简介
//! hetus 聚合了 hetu-common、hetu-core 及各功能模块，提供一站式的使用体验。
//!
//! # 使用方式
//!
//! ## 基础使用
//! ```ignore
//! use hetus::core::Application;
//! use hetus::common::time::now_offset;
//! ```
//!
//! ## Web 服务
//! ```ignore
//! use hetus::{web::Router, core::Application};
//! ```
//!
//! ## 完整功能
//! ```ignore
//! // Cargo.toml 添加
//! hetus = { version = "0.1", features = ["full"] }
//! ```

// ==================== 模块 re-export ====================

/// 基础工具模块 (hetu-common)
pub use hetu_common as common;

/// 核心框架模块 (hetu-core)
pub use hetu_core as core;

/// 宏定义模块 (hetu-core-macros)
pub use hetu_core_macros as macros;

// ==================== 功能模块 re-export ====================

#[cfg(feature = "ai")]
/// AI 模块 (hetu-ai)
pub use hetu_ai as ai;

#[cfg(feature = "db")]
/// 数据库模块 (hetu-db)
pub use hetu_db as db;

#[cfg(feature = "grpc")]
/// gRPC 模块 (hetu-grpc)
pub use hetu_grpc as grpc;

#[cfg(feature = "security")]
/// 安全认证模块 (hetu-security)
pub use hetu_security as security;

#[cfg(feature = "web")]
/// Web 框架模块 (hetu-web)
pub use hetu_web as web;

// ==================== SQL 模块 re-export ====================

#[cfg(feature = "db")]
/// SQL/ORM 模块 (hetusql)
pub use hetusql as sql;

// ==================== 子模块 ====================

#[cfg(feature = "web")]
pub mod web_utils;

// ==================== 核心类型 re-export ====================

/// 核心错误类型
pub use core::{DataError, Result};
