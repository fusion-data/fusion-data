//! 资源管理模块
//!
//! 提供工作流执行过程中的资源管理功能，包括：
//! - 资源竞争管理
//! - 死锁检测和预防
//! - 资源分配策略

mod competition_manager;
mod deadlock_detector;

pub use competition_manager::ResourceCompetitionManager;
pub use deadlock_detector::DeadlockDetector;

use fusion_common::ahash::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

/// 资源类型
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourceType {
  /// CPU 资源
  Cpu,
  /// 内存资源
  Memory(u64), // MB
  /// 文件句柄
  FileHandle,
  /// 网络连接
  NetworkConnection,
  /// 数据库连接
  DatabaseConnection,
  /// 自定义资源
  Custom(String),
}

/// 资源请求
#[derive(Debug, Clone)]
pub struct ResourceRequest {
  /// 请求ID
  pub request_id: Uuid,
  /// 请求的资源类型
  pub resource_type: ResourceType,
  /// 请求的资源数量
  pub amount: u64,
  /// 优先级
  pub priority: u8,
  /// 请求时间
  pub requested_at: chrono::DateTime<chrono::Utc>,
  /// 请求者信息
  pub requester: String,
}

/// 资源分配
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
  /// 分配ID
  pub allocation_id: Uuid,
  /// 关联的请求ID
  pub request_id: Uuid,
  /// 分配的资源类型
  pub resource_type: ResourceType,
  /// 分配的资源数量
  pub amount: u64,
  /// 分配时间
  pub allocated_at: chrono::DateTime<chrono::Utc>,
  /// 是否已释放
  pub released: bool,
}

/// 资源池
#[derive(Debug)]
pub struct ResourcePool {
  /// 资源类型
  pub resource_type: ResourceType,
  /// 总资源量
  pub total_capacity: u64,
  /// 已分配资源量
  pub allocated_amount: u64,
  /// 信号量用于控制并发访问
  pub semaphore: Arc<Semaphore>,
  /// 资源分配记录
  pub allocations: Arc<Mutex<HashMap<Uuid, ResourceAllocation>>>,
}

impl ResourcePool {
  /// 创建新的资源池
  pub fn new(resource_type: ResourceType, total_capacity: u64) -> Self {
    Self {
      semaphore: Arc::new(Semaphore::new(total_capacity as usize)),
      resource_type,
      total_capacity,
      allocated_amount: 0,
      allocations: Arc::new(Mutex::new(HashMap::default())),
    }
  }

  /// 请求资源
  pub async fn acquire(&self, request: &ResourceRequest) -> Result<ResourceAllocation, ResourceError> {
    // 检查是否有足够的资源
    if self.allocated_amount + request.amount > self.total_capacity {
      return Err(ResourceError::InsufficientResources {
        requested: request.amount,
        available: self.total_capacity - self.allocated_amount,
      });
    }

    // 获取信号量许可
    let permit = self.semaphore.acquire().await.map_err(|_| ResourceError::AcquisitionFailed)?;

    let allocation = ResourceAllocation {
      allocation_id: Uuid::now_v7(),
      request_id: request.request_id,
      resource_type: self.resource_type.clone(),
      amount: request.amount,
      allocated_at: chrono::Utc::now(),
      released: false,
    };

    // 记录分配
    {
      let mut allocations = self.allocations.lock().await;
      allocations.insert(allocation.allocation_id, allocation.clone());
    }

    // 释放许可
    drop(permit);

    Ok(allocation)
  }

  /// 释放资源
  pub async fn release(&self, allocation_id: Uuid) -> Result<(), ResourceError> {
    let mut allocations = self.allocations.lock().await;

    if let Some(allocation) = allocations.get_mut(&allocation_id) {
      if allocation.released {
        return Err(ResourceError::AlreadyReleased);
      }

      allocation.released = true;
      return Ok(());
    }

    Err(ResourceError::AllocationNotFound(allocation_id))
  }

  /// 获取可用资源量
  pub async fn available_amount(&self) -> u64 {
    let allocations = self.allocations.lock().await;
    let allocated: u64 = allocations.values().filter(|a| !a.released).map(|a| a.amount).sum();

    self.total_capacity - allocated
  }
}

/// 资源错误类型
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
  #[error("资源不足: 请求 {requested}, 可用 {available}")]
  InsufficientResources { requested: u64, available: u64 },

  #[error("资源获取失败")]
  AcquisitionFailed,

  #[error("资源分配不存在: {0}")]
  AllocationNotFound(Uuid),

  #[error("资源已释放")]
  AlreadyReleased,

  #[error("死锁检测到")]
  DeadlockDetected,

  #[error("资源配置错误: {0}")]
  ConfigurationError(String),
}

/// 资源管理器配置
#[derive(Debug, Clone)]
pub struct ResourceConfig {
  /// CPU 核心数
  pub cpu_cores: u64,
  /// 内存限制 (MB)
  pub memory_limit_mb: u64,
  /// 最大文件句柄数
  pub max_file_handles: u64,
  /// 最大网络连接数
  pub max_network_connections: u64,
  /// 最大数据库连接数
  pub max_database_connections: u64,
  /// 死锁检测间隔 (秒)
  pub deadlock_detection_interval_seconds: u64,
  /// 资源竞争检测间隔 (秒)
  pub competition_detection_interval_seconds: u64,
}

impl Default for ResourceConfig {
  fn default() -> Self {
    Self {
      cpu_cores: num_cpus::get() as u64,
      memory_limit_mb: 1024, // 1GB
      max_file_handles: 1000,
      max_network_connections: 100,
      max_database_connections: 10,
      deadlock_detection_interval_seconds: 5,
      competition_detection_interval_seconds: 1,
    }
  }
}
