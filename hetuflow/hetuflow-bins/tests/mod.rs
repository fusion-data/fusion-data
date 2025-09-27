//! hetuflow 集成测试入口
//!
//! 本文件是集成测试的主入口，包含了 hetuflow-server 与 hetuflow-agent 的完整交互测试。
//!
//! 测试覆盖以下场景：
//! - Agent 注册与心跳机制
//! - 任务轮询与获取
//! - 任务执行状态更新
//! - 任务完成与结果上报
//! - 错误处理与重试机制
//! - 并发场景处理
//! - Agent 断线重连

mod common;
mod integration;

pub use common::*;
pub use integration::*;
