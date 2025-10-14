//! 资源竞争管理器
//!
//! 负责管理多个工作流执行之间的资源竞争，提供公平的资源分配策略

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

use super::{ResourceAllocation, ResourceConfig, ResourceError, ResourcePool, ResourceRequest, ResourceType};

/// 资源竞争管理器
#[derive(Debug)]
pub struct ResourceCompetitionManager {
  /// 资源池配置
  config: ResourceConfig,
  /// 资源池映射
  resource_pools: HashMap<ResourceType, Arc<ResourcePool>>,
  /// 待处理的资源请求队列
  pending_requests: Arc<Mutex<VecDeque<ResourceRequest>>>,
  /// 资源分配通知
  allocation_notify: Arc<Notify>,
  /// 请求者优先级映射
  request_priorities: Arc<Mutex<HashMap<String, u8>>>,
  /// 统计信息
  stats: Arc<Mutex<ResourceStats>>,
}

/// 资源统计信息
#[derive(Debug, Default)]
pub struct ResourceStats {
  /// 总请求数
  pub total_requests: u64,
  /// 成功分配数
  pub successful_allocations: u64,
  /// 失败请求数
  pub failed_requests: u64,
  /// 平均等待时间（毫秒）
  pub average_wait_time_ms: u64,
  /// 资源利用率
  pub utilization_rate: f64,
}

impl ResourceCompetitionManager {
  /// 创建新的资源竞争管理器
  pub fn new(config: ResourceConfig) -> Self {
    let mut resource_pools = HashMap::new();

    // 初始化各种资源池
    resource_pools.insert(ResourceType::Cpu, Arc::new(ResourcePool::new(ResourceType::Cpu, config.cpu_cores)));
    resource_pools.insert(
      ResourceType::Memory(config.memory_limit_mb),
      Arc::new(ResourcePool::new(ResourceType::Memory(config.memory_limit_mb), config.memory_limit_mb)),
    );
    resource_pools
      .insert(ResourceType::FileHandle, Arc::new(ResourcePool::new(ResourceType::FileHandle, config.max_file_handles)));
    resource_pools.insert(
      ResourceType::NetworkConnection,
      Arc::new(ResourcePool::new(ResourceType::NetworkConnection, config.max_network_connections)),
    );
    resource_pools.insert(
      ResourceType::DatabaseConnection,
      Arc::new(ResourcePool::new(ResourceType::DatabaseConnection, config.max_database_connections)),
    );

    Self {
      config,
      resource_pools,
      pending_requests: Arc::new(Mutex::new(VecDeque::new())),
      allocation_notify: Arc::new(Notify::new()),
      request_priorities: Arc::new(Mutex::new(HashMap::new())),
      stats: Arc::new(Mutex::new(ResourceStats::default())),
    }
  }

  /// 请求资源
  pub async fn request_resource(&self, request: ResourceRequest) -> Result<ResourceAllocation, ResourceError> {
    let start_time = chrono::Utc::now();
    let request_id = request.request_id;

    // 更新统计信息
    {
      let mut stats = self.stats.lock().await;
      stats.total_requests += 1;
    }

    // 检查是否有可用的资源
    if let Some(pool) = self.resource_pools.get(&request.resource_type) {
      match pool.acquire(&request).await {
        Ok(allocation) => {
          // 成功分配
          let wait_time = chrono::Utc::now().signed_duration_since(start_time).num_milliseconds() as u64;
          self.update_success_stats(wait_time).await;
          return Ok(allocation);
        }
        Err(_) => {
          // 资源不足，加入等待队列
          self.enqueue_request(request).await;
        }
      }
    } else {
      // 未知资源类型，创建临时资源池
      return Err(ResourceError::ConfigurationError(format!("Unknown resource type: {:?}", request.resource_type)));
    }

    // 等待资源分配
    self.wait_for_allocation(&request_id).await
  }

  /// 释放资源
  pub async fn release_resource(&self, allocation_id: Uuid) -> Result<(), ResourceError> {
    // 查找并释放资源
    for pool in self.resource_pools.values() {
      if pool.release(allocation_id).await.is_ok() {
        // 通知等待的请求
        self.allocation_notify.notify_one();
        self.process_pending_requests().await;
        return Ok(());
      }
    }

    Err(ResourceError::AllocationNotFound(allocation_id))
  }

  /// 设置请求者优先级
  pub async fn set_priority(&self, requester: String, priority: u8) {
    let mut priorities = self.request_priorities.lock().await;
    priorities.insert(requester, priority);
  }

  /// 获取资源池状态
  pub async fn get_pool_status(&self, resource_type: &ResourceType) -> Option<ResourcePoolStatus> {
    if let Some(pool) = self.resource_pools.get(resource_type) {
      let available = pool.available_amount().await;
      Some(ResourcePoolStatus {
        resource_type: resource_type.clone(),
        total_capacity: pool.total_capacity,
        allocated_amount: pool.allocated_amount,
        available_amount: available,
      })
    } else {
      None
    }
  }

  /// 获取所有资源池状态
  pub async fn get_all_pool_status(&self) -> Vec<ResourcePoolStatus> {
    let mut status_vec = Vec::new();
    for (resource_type, _pool) in &self.resource_pools {
      if let Some(status) = self.get_pool_status(resource_type).await {
        status_vec.push(status);
      }
    }
    status_vec
  }

  /// 获取统计信息
  pub async fn get_stats(&self) -> ResourceStats {
    let stats = self.stats.lock().await;
    ResourceStats {
      total_requests: stats.total_requests,
      successful_allocations: stats.successful_allocations,
      failed_requests: stats.failed_requests,
      average_wait_time_ms: stats.average_wait_time_ms,
      utilization_rate: stats.utilization_rate,
    }
  }

  /// 将请求加入队列
  async fn enqueue_request(&self, request: ResourceRequest) {
    let mut queue = self.pending_requests.lock().await;
    queue.push_back(request);
  }

  /// 等待资源分配
  async fn wait_for_allocation(&self, request_id: &Uuid) -> Result<ResourceAllocation, ResourceError> {
    loop {
      // 检查是否有分配结果
      {
        let queue = self.pending_requests.lock().await;
        if let Some(request) = queue.iter().find(|r| &r.request_id == request_id) {
          if let Some(pool) = self.resource_pools.get(&request.resource_type) {
            match pool.acquire(request).await {
              Ok(allocation) => return Ok(allocation),
              Err(_) => {
                // 继续等待
                drop(queue);
                self.allocation_notify.notified().await;
              }
            }
          }
        }
      }
    }
  }

  /// 处理待处理的请求
  async fn process_pending_requests(&self) {
    let mut queue = self.pending_requests.lock().await;
    let mut to_remove = Vec::new();

    for (index, request) in queue.iter().enumerate() {
      if let Some(pool) = self.resource_pools.get(&request.resource_type) {
        if pool.available_amount().await >= request.amount {
          // 尝试分配资源
          match pool.acquire(request).await {
            Ok(_) => {
              to_remove.push(index);
              // 更新成功统计
              let mut stats = self.stats.lock().await;
              stats.successful_allocations += 1;
            }
            Err(_) => {
              // 记录失败统计
              let mut stats = self.stats.lock().await;
              stats.failed_requests += 1;
            }
          }
        }
      }
    }

    // 从队列中移除已处理的请求
    for &index in to_remove.iter().rev() {
      queue.remove(index);
    }
  }

  /// 更新成功统计
  async fn update_success_stats(&self, wait_time: u64) {
    let mut stats = self.stats.lock().await;
    stats.successful_allocations += 1;

    // 更新平均等待时间
    if stats.successful_allocations > 0 {
      stats.average_wait_time_ms =
        (stats.average_wait_time_ms * (stats.successful_allocations - 1) + wait_time) / stats.successful_allocations;
    }

    // 计算资源利用率
    let mut total_allocated = 0u64;
    let mut total_capacity = 0u64;

    for pool in self.resource_pools.values() {
      total_allocated += pool.allocated_amount;
      total_capacity += pool.total_capacity;
    }

    if total_capacity > 0 {
      stats.utilization_rate = total_allocated as f64 / total_capacity as f64;
    }
  }

  /// 启动资源竞争监控
  pub async fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
    let pending_requests = self.pending_requests.clone();
    let allocation_notify = self.allocation_notify.clone();
    let resource_pools = self.resource_pools.clone();

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(
        std::time::Duration::from_secs(1), // 每秒检查一次
      );

      loop {
        tokio::select! {
            _ = interval.tick() => {
                // 定期处理待处理请求
                let queue_len = pending_requests.lock().await.len();
                if queue_len > 0 {
                    allocation_notify.notify_one();
                }
            }
            _ = allocation_notify.notified() => {
                // 有资源释放时立即处理
                Self::process_queue(&pending_requests, &resource_pools).await;
            }
        }
      }
    })
  }

  /// 处理队列的静态方法
  async fn process_queue(
    pending_requests: &Arc<Mutex<VecDeque<ResourceRequest>>>,
    resource_pools: &HashMap<ResourceType, Arc<ResourcePool>>,
  ) {
    let mut queue = pending_requests.lock().await;
    let mut to_remove = Vec::new();

    for (index, request) in queue.iter().enumerate() {
      if let Some(pool) = resource_pools.get(&request.resource_type) {
        if pool.available_amount().await >= request.amount {
          // 尝试分配资源
          if pool.acquire(request).await.is_ok() {
            to_remove.push(index);
          }
        }
      }
    }

    // 从队列中移除已处理的请求
    for &index in to_remove.iter().rev() {
      queue.remove(index);
    }
  }
}

/// 资源池状态
#[derive(Debug, Clone)]
pub struct ResourcePoolStatus {
  /// 资源类型
  pub resource_type: ResourceType,
  /// 总容量
  pub total_capacity: u64,
  /// 已分配量
  pub allocated_amount: u64,
  /// 可用量
  pub available_amount: u64,
}

impl ResourcePoolStatus {
  /// 获取利用率（0.0 - 1.0）
  pub fn utilization_rate(&self) -> f64 {
    if self.total_capacity == 0 { 0.0 } else { self.allocated_amount as f64 / self.total_capacity as f64 }
  }
}
