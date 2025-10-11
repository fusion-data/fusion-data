//! 二进制数据存储模块
//!
//! 本模块提供了基于 opendal 的二进制数据引用系统，包括：
//! - 存储抽象层
//! - 生命周期管理
//! - 基础指标收集

pub mod config;
pub mod error;
pub mod lifecycle;
pub mod manager;
pub mod metadata;
pub mod metrics;
pub mod storage;

// 重新导出主要类型
pub use config::{BinaryStorageConfig, StorageType};
pub use error::BinaryStorageError;
pub use lifecycle::{BinaryDataLifecycleManager, LifecycleCleanupConfig};
pub use manager::BinaryDataManager;
pub use metadata::BinaryDataMetadata;
pub use metrics::{BasicMetricsCollector, BasicStats, OperationProgress};
pub use storage::{BinaryDataStorage, StorageCapabilities};
