//! 二进制数据存储实现
//!
//! 本模块提供了基于 opendal 的二进制数据存储的具体实现。

pub mod manager_factory;
pub mod opendal_storage;

// 重新导出主要类型
pub use manager_factory::*;
pub use opendal_storage::*;
