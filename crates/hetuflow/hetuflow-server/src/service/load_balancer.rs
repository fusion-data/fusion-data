use std::collections::HashMap;
use std::sync::Arc;

use fusion_common::time::{OffsetDateTime, now_offset};
use fusion_core::DataError;
use log::{debug, info, warn};
use modelsql::ModelManager;
use uuid::Uuid;

use crate::infra::bmc::*;
use hetuflow_core::models::*;

/// 负载均衡缓存
#[derive(Debug, Clone)]
struct LoadBalanceCache {
  servers: HashMap<String, ServerLoadInfo>,
  last_updated: OffsetDateTime,
}

/// 服务器负载信息
#[derive(Debug, Clone)]
struct ServerLoadInfo {
  server: SchedServer,
  active_tasks: u32,
  load_score: f64,
}

/// 重平衡触发条件
#[derive(Debug, Clone)]
pub struct RebalanceThreshold {
  /// 负载差异触发重平衡阈值，百分比
  pub load_variance_threshold: f64,
  /// 最小重平衡间隔（分钟）
  pub min_rebalance_interval_minutes: i64,
}

impl Default for RebalanceThreshold {
  fn default() -> Self {
    Self {
      load_variance_threshold: 0.3,       // 30% 负载差异触发重平衡
      min_rebalance_interval_minutes: 10, // 最小重平衡间隔
    }
  }
}

/// 负载均衡器 - 负责服务器间的负载均衡和命名空间分配。只有 Leader 节点可以执行负载均衡操作。
pub struct LoadBalancer {
  mm: ModelManager,
  server_id: String,
  // 负载均衡策略缓存
  balance_cache: Arc<tokio::sync::RwLock<LoadBalanceCache>>,
}

impl LoadBalancer {
  /// 创建新的负载均衡器
  pub fn new(mm: ModelManager, server_id: String) -> Self {
    Self {
      mm,
      server_id,
      balance_cache: Arc::new(tokio::sync::RwLock::new(LoadBalanceCache {
        servers: HashMap::default(),
        last_updated: now_offset(),
      })),
    }
  }

  /// 检查是否需要重平衡并执行
  pub async fn rebalance_if_needed(&self) -> Result<(), DataError> {
    log::trace!("Checking if rebalancing is needed");

    // 更新服务器负载信息
    self.update_server_load_cache().await?;

    // 检查是否需要重平衡
    if self.should_rebalance().await? {
      info!("Load imbalance detected, starting rebalancing");
      self.perform_rebalance().await?;
    } else {
      log::trace!("No rebalancing needed");
    }

    Ok(())
  }

  /// 更新服务器负载缓存
  async fn update_server_load_cache(&self) -> Result<(), DataError> {
    let servers = ServerBmc::find_active_servers(&self.mm).await?;
    let mut new_cache = LoadBalanceCache { servers: HashMap::default(), last_updated: now_offset() };

    for server in servers {
      let load_info = self.calculate_server_load(&server).await?;
      new_cache.servers.insert(server.id, load_info);
    }

    *self.balance_cache.write().await = new_cache;

    log::trace!("Updated load balance cache with {} servers", self.balance_cache.read().await.servers.len());
    Ok(())
  }

  /// 计算服务器负载
  async fn calculate_server_load(&self, server: &SchedServer) -> Result<ServerLoadInfo, DataError> {
    let active_tasks = TaskBmc::count_active_tasks_by_server(&self.mm, server.id.clone()).await? as u32;
    let load_score = self.calculate_load_score(active_tasks);

    Ok(ServerLoadInfo { server: server.clone(), active_tasks, load_score })
  }

  /// 计算负载评分
  fn calculate_load_score(&self, active_tasks: u32) -> f64 {
    let task_weight = 1.0;
    active_tasks as f64 * task_weight
  }

  /// 检查是否需要重平衡
  async fn should_rebalance(&self) -> Result<bool, DataError> {
    let cache = self.balance_cache.read().await;
    let threshold = RebalanceThreshold::default();

    // 检查最小重平衡间隔
    let time_since_last_update = now_offset() - cache.last_updated;
    if time_since_last_update.num_minutes() < threshold.min_rebalance_interval_minutes {
      return Ok(false);
    }

    // 检查服务器数量
    if cache.servers.len() < 2 {
      return Ok(false);
    }

    // 计算负载方差
    let load_scores: Vec<f64> = cache.servers.values().map(|info| info.load_score).collect();
    let mean_load = load_scores.iter().sum::<f64>() / load_scores.len() as f64;

    let variance = load_scores.iter().map(|score| (score - mean_load).powi(2)).sum::<f64>() / load_scores.len() as f64;

    let coefficient_of_variation = if mean_load > 0.0 { variance.sqrt() / mean_load } else { 0.0 };

    debug!(
      "Load variance coefficient: {:.3}, threshold: {:.3}",
      coefficient_of_variation, threshold.load_variance_threshold
    );

    Ok(coefficient_of_variation > threshold.load_variance_threshold)
  }

  /// 执行重平衡（基于负载的动态调度，不做 namespace_id 迁移）
  async fn perform_rebalance(&self) -> Result<(), DataError> {
    info!("Starting load rebalancing (dynamic)");

    // 更新服务器缓存
    self.update_server_load_cache().await?;

    // 更新 Server namespace_id 绑定
    self.update_server_namespace_bind().await?;

    Ok(())
  }

  /// 获取负载均衡状态
  pub async fn get_balance_status(&self) -> Result<serde_json::Value, DataError> {
    self.update_server_load_cache().await?;

    let cache = self.balance_cache.read().await;

    let servers: Vec<serde_json::Value> = cache
      .servers
      .values()
      .map(|info| {
        serde_json::json!({
          "server_id": info.server.id,
          "server_name": info.server.name,
          "active_tasks": info.active_tasks,
          "load_score": info.load_score,
        })
      })
      .collect();

    Ok(serde_json::json!({
      "servers": servers,
      "last_updated": cache.last_updated,
      "total_servers": cache.servers.len(),
    }))
  }

  /// 获取统计信息
  pub async fn get_stats(&self) -> serde_json::Value {
    let cache = self.balance_cache.read().await;

    let total_tasks: u32 = cache.servers.values().map(|info| info.active_tasks).sum();
    let load_scores: Vec<f64> = cache.servers.values().map(|info| info.load_score).collect();
    let avg_load =
      if !load_scores.is_empty() { load_scores.iter().sum::<f64>() / load_scores.len() as f64 } else { 0.0 };

    serde_json::json!({
      "server_id": self.server_id,
      "total_servers": cache.servers.len(),
      "total_tasks": total_tasks,

      "average_load": avg_load,
      "last_updated": cache.last_updated,
    })
  }

  /// 更新服务器 namespace_id 绑定
  async fn update_server_namespace_bind(&self) -> Result<(), DataError> {
    info!("Updating server namespace_id bindings based on load balance");

    // 获取所有活跃的 namespaces
    let all_namespaces = self.get_all_active_namespaces().await?;

    if all_namespaces.is_empty() {
      debug!("No active namespaces found");
      return Ok(());
    }

    // 获取当前负载缓存
    let cache = self.balance_cache.read().await;
    let mut servers: Vec<_> = cache.servers.values().collect();

    if servers.is_empty() {
      warn!("No active servers found for namespace_id binding");
      return Ok(());
    }

    // 按负载评分排序（负载低的优先）
    servers.sort_by(|a, b| a.load_score.partial_cmp(&b.load_score).unwrap_or(std::cmp::Ordering::Equal));

    // 计算每个服务器应该分配的 namespace_id 数量
    let namespaces_per_server = all_namespaces.len().div_ceil(servers.len()); // 向上取整

    // 分配 namespaces 到服务器
    let mut namespace_assignments: HashMap<String, Vec<Uuid>> = HashMap::default();

    for (i, namespace_id) in all_namespaces.iter().enumerate() {
      let server_index = i / namespaces_per_server;
      let server_index = server_index.min(servers.len() - 1); // 确保不越界
      let server_id = servers[server_index].server.id.clone();

      namespace_assignments.entry(server_id).or_default().push(*namespace_id);
    }

    // 更新数据库中的绑定关系
    for (server_id, namespaces) in &namespace_assignments {
      ServerBmc::update_server_namespace_bind(&self.mm, server_id, namespaces.clone())
        .await
        .map_err(|e| DataError::server_error(format!("Failed to update server namespace_id bind: {}", e)))?;

      info!("Updated server {} with {} namespaces", server_id, namespaces.len());
    }

    // 清空没有分配到 namespace_id 的服务器
    for server_info in servers {
      if namespace_assignments.contains_key(&server_info.server.id) {
        continue;
      }
      ServerBmc::update_server_namespace_bind(&self.mm, server_info.server.id.as_str(), vec![])
        .await
        .map_err(|e| DataError::server_error(format!("Failed to clear server namespace_id bind: {}", e)))?;
      debug!("Cleared namespaces for server {}", server_info.server.id);
    }

    Ok(())
  }

  /// 获取所有活跃的 namespaces
  async fn get_all_active_namespaces(&self) -> Result<Vec<Uuid>, DataError> {
    // 从任务表中获取所 namespace_id。只获取未取消及未完成的 < TaskStatus::Failed，对于错误完成的任务因可能被重试，所以任被考虑。
    let sql =
      "select distinct namespace_id from sched_task where status < 90 or (status = 90 and retry_count < max_retries)";
    let db = self.mm.dbx().db_postgres().map_err(|e| DataError::server_error(format!("Database error: {}", e)))?;
    let query = sqlx::query_as(sql);
    let rows: Vec<(Uuid,)> = db.fetch_all(query).await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rebalance_threshold() {
    let threshold = RebalanceThreshold::default();
    assert_eq!(threshold.load_variance_threshold, 0.3);
    assert_eq!(threshold.min_rebalance_interval_minutes, 10);
  }

  #[test]
  fn test_load_balance_cache() {
    let cache = LoadBalanceCache { servers: HashMap::default(), last_updated: now_offset() };

    assert!(cache.servers.is_empty());
  }
}
