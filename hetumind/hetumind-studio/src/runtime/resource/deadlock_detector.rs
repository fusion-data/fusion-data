//! 死锁检测器
//!
//! 实现资源分配图和死锁检测算法，预防和解决死锁问题

use fusion_common::ahash::HashSet;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

use super::{ResourceAllocation, ResourceError, ResourceRequest, ResourceType};

/// 死锁检测器
#[derive(Debug)]
pub struct DeadlockDetector {
  /// 资源分配图
  allocation_graph: Arc<Mutex<ResourceAllocationGraph>>,
  /// 死锁检测通知
  deadlock_notify: Arc<Notify>,
  /// 死锁统计
  stats: Arc<Mutex<DeadlockStats>>,
  /// 是否启用死锁检测
  enabled: Arc<Mutex<bool>>,
}

/// 资源分配图
#[derive(Debug, Default, Clone)]
pub struct ResourceAllocationGraph {
  /// 进程节点（工作流执行实例）
  processes: HashMap<String, ProcessNode>,
  /// 资源节点
  resources: HashMap<ResourceType, ResourceNode>,
  /// 分配边：进程 -> 资源
  allocation_edges: HashMap<String, HashSet<ResourceType>>,
  /// 请求边：进程 -> 资源
  request_edges: HashMap<String, HashSet<ResourceType>>,
}

/// 进程节点
#[derive(Debug, Clone)]
pub struct ProcessNode {
  /// 进程ID
  pub process_id: String,
  /// 进程状态
  pub status: ProcessStatus,
  /// 持有的资源
  pub held_resources: HashSet<ResourceType>,
  /// 等待的资源
  pub waiting_resources: HashSet<ResourceType>,
  /// 优先级
  pub priority: u8,
  /// 创建时间
  pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 资源节点
#[derive(Debug, Clone)]
pub struct ResourceNode {
  /// 资源类型
  pub resource_type: ResourceType,
  /// 总数量
  pub total_amount: u64,
  /// 已分配数量
  pub allocated_amount: u64,
  /// 可用数量
  pub available_amount: u64,
  /// 等待队列
  pub waiting_queue: VecDeque<String>,
}

/// 进程状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
  /// 运行中
  Running,
  /// 等待资源
  Waiting,
  /// 被阻塞
  Blocked,
  /// 已完成
  Completed,
}

/// 死锁检测结果
#[derive(Debug, Clone)]
pub struct DeadlockDetectionResult {
  /// 是否存在死锁
  pub has_deadlock: bool,
  /// 死锁的进程集合
  pub deadlocked_processes: HashSet<String>,
  /// 涉及的资源集合
  pub involved_resources: HashSet<ResourceType>,
  /// 检测时间
  pub detected_at: chrono::DateTime<chrono::Utc>,
  /// 建议的解决方案
  pub suggested_actions: Vec<DeadlockResolution>,
}

/// 死锁解决方案
#[derive(Debug, Clone)]
pub enum DeadlockResolution {
  /// 强制终止进程
  TerminateProcess { process_id: String },
  /// 抢占资源
  PreemptResource { process_id: String, resource_type: ResourceType },
  /// 优先级调度
  PriorityScheduling { processes: Vec<String> },
}

/// 死锁统计信息
#[derive(Debug, Default, Clone)]
pub struct DeadlockStats {
  /// 检测次数
  pub detection_count: u64,
  /// 死锁次数
  pub deadlock_count: u64,
  /// 解决次数
  pub resolution_count: u64,
  /// 平均检测时间（毫秒）
  pub average_detection_time_ms: u64,
}

impl DeadlockDetector {
  /// 创建新的死锁检测器
  pub fn new() -> Self {
    Self {
      allocation_graph: Arc::new(Mutex::new(ResourceAllocationGraph::default())),
      deadlock_notify: Arc::new(Notify::new()),
      stats: Arc::new(Mutex::new(DeadlockStats::default())),
      enabled: Arc::new(Mutex::new(true)),
    }
  }

  /// 启用/禁用死锁检测
  pub async fn set_enabled(&self, enabled: bool) {
    let mut enabled_lock = self.enabled.lock().await;
    *enabled_lock = enabled;
  }

  /// 添加进程节点
  pub async fn add_process(&self, process_id: String, priority: u8) -> Result<(), ResourceError> {
    let mut graph = self.allocation_graph.lock().await;

    let process = ProcessNode {
      process_id: process_id.clone(),
      status: ProcessStatus::Running,
      held_resources: HashSet::default(),
      waiting_resources: HashSet::default(),
      priority,
      created_at: chrono::Utc::now(),
    };

    graph.processes.insert(process_id.clone(), process);
    graph.allocation_edges.insert(process_id.clone(), HashSet::default());
    graph.request_edges.insert(process_id, HashSet::default());

    Ok(())
  }

  /// 移除进程节点
  pub async fn remove_process(&self, process_id: &str) -> Result<(), ResourceError> {
    let mut graph = self.allocation_graph.lock().await;

    // 释放所有持有的资源 - 先收集资源类型，避免借用冲突
    let held_resources: Vec<ResourceType> = if let Some(process) = graph.processes.get(process_id) {
      process.held_resources.iter().cloned().collect()
    } else {
      Vec::new()
    };

    for resource_type in &held_resources {
      if let Some(resource) = graph.resources.get_mut(resource_type) {
        resource.allocated_amount = resource.allocated_amount.saturating_sub(1);
        resource.available_amount += 1;
      }
    }

    // 移除进程
    graph.processes.remove(process_id);
    graph.allocation_edges.remove(process_id);
    graph.request_edges.remove(process_id);

    // 从资源等待队列中移除
    for resource in graph.resources.values_mut() {
      resource.waiting_queue.retain(|p| p != process_id);
    }

    Ok(())
  }

  /// 添加资源节点
  pub async fn add_resource(&self, resource_type: ResourceType, total_amount: u64) -> Result<(), ResourceError> {
    let mut graph = self.allocation_graph.lock().await;

    let resource = ResourceNode {
      resource_type: resource_type.clone(),
      total_amount,
      allocated_amount: 0,
      available_amount: total_amount,
      waiting_queue: VecDeque::new(),
    };

    graph.resources.insert(resource_type, resource);
    Ok(())
  }

  /// 记录资源分配
  pub async fn record_allocation(&self, process_id: &str, resource_type: &ResourceType) -> Result<(), ResourceError> {
    let mut graph = self.allocation_graph.lock().await;

    // 更新进程状态
    if let Some(process) = graph.processes.get_mut(process_id) {
      process.held_resources.insert(resource_type.clone());
      process.waiting_resources.remove(resource_type);
      if process.waiting_resources.is_empty() {
        process.status = ProcessStatus::Running;
      }
    }

    // 更新资源状态
    if let Some(resource) = graph.resources.get_mut(resource_type) {
      resource.allocated_amount += 1;
      resource.available_amount -= 1;
    }

    // 添加分配边
    if let Some(allocations) = graph.allocation_edges.get_mut(process_id) {
      allocations.insert(resource_type.clone());
    }

    // 移除请求边
    if let Some(requests) = graph.request_edges.get_mut(process_id) {
      requests.remove(resource_type);
    }

    Ok(())
  }

  /// 记录资源请求
  pub async fn record_request(&self, process_id: &str, resource_type: &ResourceType) -> Result<(), ResourceError> {
    let mut graph = self.allocation_graph.lock().await;

    // 更新进程状态
    if let Some(process) = graph.processes.get_mut(process_id) {
      process.waiting_resources.insert(resource_type.clone());
      process.status = ProcessStatus::Waiting;
    }

    // 更新资源等待队列
    if let Some(resource) = graph.resources.get_mut(resource_type) {
      if !resource.waiting_queue.contains(&process_id.to_string()) {
        resource.waiting_queue.push_back(process_id.to_string());
      }
    }

    // 添加请求边
    if let Some(requests) = graph.request_edges.get_mut(process_id) {
      requests.insert(resource_type.clone());
    }

    // 触发死锁检测
    self.deadlock_notify.notify_one();
    Ok(())
  }

  /// 记录资源释放
  pub async fn record_release(&self, process_id: &str, resource_type: &ResourceType) -> Result<(), ResourceError> {
    let waiting_process = {
      let mut graph = self.allocation_graph.lock().await;

      // 更新进程状态
      if let Some(process) = graph.processes.get_mut(process_id) {
        process.held_resources.remove(resource_type);
      }

      // 更新资源状态
      if let Some(resource) = graph.resources.get_mut(resource_type) {
        resource.allocated_amount = resource.allocated_amount.saturating_sub(1);
        resource.available_amount += 1;

        // 获取等待队列中的进程
        resource.waiting_queue.pop_front()
      } else {
        None
      }
    };

    // 如果有等待的进程，为其分配资源
    if let Some(waiting_process) = waiting_process {
      return self.record_allocation(&waiting_process, resource_type).await;
    }

    // 移除分配边
    {
      let mut graph = self.allocation_graph.lock().await;
      if let Some(allocations) = graph.allocation_edges.get_mut(process_id) {
        allocations.remove(resource_type);
      }
    }

    Ok(())
  }

  /// 执行死锁检测
  pub async fn detect_deadlock(&self) -> DeadlockDetectionResult {
    let start_time = std::time::Instant::now();
    let mut stats = self.stats.lock().await;
    stats.detection_count += 1;
    drop(stats);

    let graph = self.allocation_graph.lock().await;
    let deadlocked_processes = self.find_deadlocked_processes(&graph);

    let has_deadlock = !deadlocked_processes.is_empty();
    let involved_resources = self.find_involved_resources(&graph, &deadlocked_processes);

    let suggested_actions =
      if has_deadlock { self.suggest_resolutions(&graph, &deadlocked_processes) } else { Vec::new() };

    let detection_time = start_time.elapsed().as_millis() as u64;

    // 更新统计
    {
      let mut stats = self.stats.lock().await;
      if has_deadlock {
        stats.deadlock_count += 1;
      }

      if stats.detection_count > 0 {
        stats.average_detection_time_ms =
          (stats.average_detection_time_ms * (stats.detection_count - 1) + detection_time) / stats.detection_count;
      }
    }

    DeadlockDetectionResult {
      has_deadlock,
      deadlocked_processes,
      involved_resources,
      detected_at: chrono::Utc::now(),
      suggested_actions,
    }
  }

  /// 查找死锁进程（使用资源分配图算法）
  fn find_deadlocked_processes(&self, graph: &ResourceAllocationGraph) -> HashSet<String> {
    let mut deadlocked = HashSet::default();
    let mut visited = HashSet::default();

    // 对每个等待资源的进程进行DFS
    for (process_id, process) in &graph.processes {
      if process.status == ProcessStatus::Waiting && !visited.contains(process_id) {
        if self.is_in_deadlock_cycle(graph, process_id, &mut visited) {
          deadlocked.insert(process_id.clone());
        }
      }
    }

    deadlocked
  }

  /// 检查进程是否在死锁环中
  fn is_in_deadlock_cycle(
    &self,
    graph: &ResourceAllocationGraph,
    process_id: &str,
    visited: &mut HashSet<String>,
  ) -> bool {
    if visited.contains(process_id) {
      return true; // 发现环
    }

    visited.insert(process_id.to_string());

    // 检查该进程等待的资源
    if let Some(waiting_resources) = graph.request_edges.get(process_id) {
      for resource_type in waiting_resources {
        // 找到持有该资源的其他进程
        for (other_process_id, other_process) in &graph.processes {
          if other_process.held_resources.contains(resource_type) {
            if self.is_in_deadlock_cycle(graph, other_process_id, visited) {
              return true;
            }
          }
        }
      }
    }

    visited.remove(process_id);
    false
  }

  /// 查找涉及的资源
  fn find_involved_resources(
    &self,
    graph: &ResourceAllocationGraph,
    deadlocked_processes: &HashSet<String>,
  ) -> HashSet<ResourceType> {
    let mut involved_resources = HashSet::default();

    for process_id in deadlocked_processes {
      if let Some(process) = graph.processes.get(process_id) {
        involved_resources.extend(process.waiting_resources.clone());
        involved_resources.extend(process.held_resources.clone());
      }
    }

    involved_resources
  }

  /// 建议死锁解决方案
  fn suggest_resolutions(
    &self,
    graph: &ResourceAllocationGraph,
    deadlocked_processes: &HashSet<String>,
  ) -> Vec<DeadlockResolution> {
    let mut resolutions = Vec::new();

    // 找到优先级最低的进程
    let mut lowest_priority_process = None;
    let mut lowest_priority = u8::MAX;

    for process_id in deadlocked_processes {
      if let Some(process) = graph.processes.get(process_id) {
        if process.priority < lowest_priority {
          lowest_priority = process.priority;
          lowest_priority_process = Some(process_id.clone());
        }
      }
    }

    // 建议终止最低优先级进程
    if let Some(process_id) = lowest_priority_process {
      resolutions.push(DeadlockResolution::TerminateProcess { process_id });
    }

    // 建议抢占资源
    for process_id in deadlocked_processes {
      if let Some(process) = graph.processes.get(process_id) {
        if !process.held_resources.is_empty() {
          let resource_type = process.held_resources.iter().next().unwrap().clone();
          resolutions.push(DeadlockResolution::PreemptResource { process_id: process_id.clone(), resource_type });
        }
      }
    }

    // 建议优先级调度
    let mut processes: Vec<_> = deadlocked_processes.iter().cloned().collect();
    processes.sort_by(|a, b| {
      let priority_a = graph.processes.get(a).map(|p| p.priority).unwrap_or(0);
      let priority_b = graph.processes.get(b).map(|p| p.priority).unwrap_or(0);
      priority_b.cmp(&priority_a) // 高优先级在前
    });

    if !processes.is_empty() {
      resolutions.push(DeadlockResolution::PriorityScheduling { processes });
    }

    resolutions
  }

  /// 执行死锁解决
  pub async fn resolve_deadlock(&self, resolution: &DeadlockResolution) -> Result<(), ResourceError> {
    match resolution {
      DeadlockResolution::TerminateProcess { process_id } => {
        self.remove_process(process_id).await?;
        let mut stats = self.stats.lock().await;
        stats.resolution_count += 1;
      }
      DeadlockResolution::PreemptResource { process_id, resource_type } => {
        self.record_release(process_id, resource_type).await?;
        let mut stats = self.stats.lock().await;
        stats.resolution_count += 1;
      }
      DeadlockResolution::PriorityScheduling { processes } => {
        // 优先级调度需要外部协调执行
        log::info!("Priority scheduling requested for processes: {:?}", processes);
      }
    }
    Ok(())
  }

  /// 启动死锁检测监控
  pub async fn start_monitoring(&self, interval_seconds: u64) -> tokio::task::JoinHandle<()> {
    let deadlock_notify = self.deadlock_notify.clone();
    let stats = self.stats.clone();
    let enabled = self.enabled.clone();
    let allocation_graph = self.allocation_graph.clone();

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));

      loop {
        tokio::select! {
            _ = interval.tick() => {
                // 定期检测
                if *enabled.lock().await {
                    let detector = DeadlockDetector {
                        allocation_graph: allocation_graph.clone(),
                        deadlock_notify: deadlock_notify.clone(),
                        stats: stats.clone(),
                        enabled: enabled.clone(),
                    };

                    let result = detector.detect_deadlock().await;
                    if result.has_deadlock {
                        log::warn!("Deadlock detected: {:?}", result);

                        // 自动解决死锁（可选）
                        if let Some(first_resolution) = result.suggested_actions.first() {
                            if let Err(e) = detector.resolve_deadlock(first_resolution).await {
                                log::error!("Failed to resolve deadlock: {:?}", e);
                            }
                        }
                    }
                }
            }
            _ = deadlock_notify.notified() => {
                // 资源状态变化时立即检测
                if *enabled.lock().await {
                    let detector = DeadlockDetector {
                        allocation_graph: allocation_graph.clone(),
                        deadlock_notify: deadlock_notify.clone(),
                        stats: stats.clone(),
                        enabled: enabled.clone(),
                    };

                    let result = detector.detect_deadlock().await;
                    if result.has_deadlock {
                        log::warn!("Deadlock detected after state change: {:?}", result);
                    }
                }
            }
        }
      }
    })
  }

  /// 获取死锁统计信息
  pub async fn get_stats(&self) -> DeadlockStats {
    let stats = self.stats.lock().await;
    DeadlockStats {
      detection_count: stats.detection_count,
      deadlock_count: stats.deadlock_count,
      resolution_count: stats.resolution_count,
      average_detection_time_ms: stats.average_detection_time_ms,
    }
  }

  /// 获取当前资源分配图快照
  pub async fn get_graph_snapshot(&self) -> ResourceAllocationGraph {
    self.allocation_graph.lock().await.clone()
  }
}

impl Default for DeadlockDetector {
  fn default() -> Self {
    Self::new()
  }
}
