use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use hetumind_core::workflow::{ConnectionKind, ExecutionData, NodeExecutionContext, NodeExecutionError};
use tokio::sync::RwLock;

/// 连接数据缓存项
#[derive(Debug, Clone)]
pub struct ConnectionCacheItem {
  pub data: ExecutionData,
  pub timestamp: chrono::DateTime<chrono::Utc>,
  pub ttl_seconds: u64,
}

impl ConnectionCacheItem {
  /// 创建新的缓存项
  pub fn new(data: ExecutionData, ttl_seconds: u64) -> Self {
    Self { data, timestamp: chrono::Utc::now(), ttl_seconds }
  }

  /// 检查缓存项是否过期
  pub fn is_expired(&self) -> bool {
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(self.timestamp);
    age.num_seconds() > self.ttl_seconds as i64
  }
}

/// 连接数据管理器
pub struct ConnectionDataManager {
  /// 连接数据缓存
  cache: Arc<RwLock<HashMap<String, ConnectionCacheItem>>>,
  /// 默认缓存TTL（秒）
  default_ttl: u64,
  /// 最大缓存条目数
  max_cache_size: usize,
}

impl ConnectionDataManager {
  /// 创建新的连接数据管理器
  pub fn new(default_ttl_seconds: u64, max_cache_size: usize) -> Self {
    Self { cache: Arc::new(RwLock::new(HashMap::new())), default_ttl: default_ttl_seconds, max_cache_size }
  }

  /// 生成缓存键
  fn generate_cache_key(
    &self,
    connection_type: &ConnectionKind,
    index: usize,
    context: &NodeExecutionContext,
  ) -> Result<String, NodeExecutionError> {
    // 基于连接类型、索引和上下文生成唯一键
    let context_id = format!("{:?}", context.execution_id);
    let connection_id = format!("{:?}", connection_type);
    Ok(format!("{}:{}:{}", connection_id, index, context_id))
  }

  /// 获取连接数据（带缓存）
  pub async fn get_connection_data(
    &self,
    context: &NodeExecutionContext,
    connection_type: ConnectionKind,
    index: usize,
  ) -> Result<Option<ExecutionData>, NodeExecutionError> {
    let cache_key = self.generate_cache_key(&connection_type, index, context)?;

    // 1. 检查缓存
    {
      let cache = self.cache.read().await;
      if let Some(cache_item) = cache.get(&cache_key) {
        if !cache_item.is_expired() {
          return Ok(Some(cache_item.data.clone()));
        }
      }
    }

    // 2. 缓存未命中或已过期，从上下文获取
    let data = context.get_connection_data(connection_type, index);

    // 3. 如果有数据，更新缓存
    if let Some(ref data) = data {
      self.update_cache(cache_key, data.clone()).await?;
    }

    Ok(data)
  }

  /// 批量获取连接数据
  pub async fn get_all_connections(
    &self,
    context: &NodeExecutionContext,
    connection_type: ConnectionKind,
  ) -> Result<Vec<ExecutionData>, NodeExecutionError> {
    let mut connections = Vec::new();
    let mut index = 0;

    // 逐个获取连接，直到没有更多连接
    while let Some(data) = self.get_connection_data(context, connection_type, index).await? {
      connections.push(data);
      index += 1;
    }

    Ok(connections)
  }

  /// 更新缓存
  async fn update_cache(&self, key: String, data: ExecutionData) -> Result<(), NodeExecutionError> {
    let mut cache = self.cache.write().await;

    // 如果缓存已满，清理过期项
    if cache.len() >= self.max_cache_size {
      self.cleanup_expired_cache(&mut cache).await;
    }

    // 如果仍然满了，删除最旧的条目
    if cache.len() >= self.max_cache_size {
      self.evict_oldest_entry(&mut cache).await;
    }

    cache.insert(key, ConnectionCacheItem::new(data, self.default_ttl));
    Ok(())
  }

  /// 清理过期的缓存条目
  async fn cleanup_expired_cache(&self, cache: &mut HashMap<String, ConnectionCacheItem>) {
    let now = chrono::Utc::now();
    cache.retain(|_, item| {
      let age = now.signed_duration_since(item.timestamp);
      age.num_seconds() <= item.ttl_seconds as i64
    });
  }

  /// 删除最旧的缓存条目
  async fn evict_oldest_entry(&self, cache: &mut HashMap<String, ConnectionCacheItem>) {
    if let Some((oldest_key, _)) = cache.iter().min_by_key(|(_, item)| item.timestamp) {
      let oldest_key = oldest_key.clone();
      cache.remove(&oldest_key);
    }
  }

  /// 清空所有缓存
  pub async fn clear_cache(&self) {
    let mut cache = self.cache.write().await;
    cache.clear();
  }

  /// 获取缓存统计信息
  pub async fn get_cache_stats(&self) -> ConnectionCacheStats {
    let cache = self.cache.read().await;
    let now = chrono::Utc::now();

    let total_entries = cache.len();
    let expired_entries = cache
      .values()
      .filter(|item| {
        let age = now.signed_duration_since(item.timestamp);
        age.num_seconds() > item.ttl_seconds as i64
      })
      .count();

    ConnectionCacheStats {
      total_entries,
      expired_entries,
      active_entries: total_entries - expired_entries,
      max_capacity: self.max_cache_size,
    }
  }
}

/// 连接缓存统计信息
#[derive(Debug, Clone)]
pub struct ConnectionCacheStats {
  pub total_entries: usize,
  pub expired_entries: usize,
  pub active_entries: usize,
  pub max_capacity: usize,
}

impl Default for ConnectionDataManager {
  fn default() -> Self {
    Self::new(300, 1000) // 默认5分钟TTL，最多1000个缓存项
  }
}

/// 连接数据管理器单例
static CONNECTION_MANAGER: once_cell::sync::Lazy<Arc<ConnectionDataManager>> =
  once_cell::sync::Lazy::new(|| Arc::new(ConnectionDataManager::default()));

/// 获取全局连接数据管理器
pub fn get_connection_manager() -> Arc<ConnectionDataManager> {
  Arc::clone(&CONNECTION_MANAGER)
}

/// 优化的连接数据获取trait
pub trait OptimizedConnectionContext {
  /// 优化版本的获取连接数据
  fn get_connection_data_optimized(
    &self,
    connection_type: ConnectionKind,
    index: usize,
  ) -> impl std::future::Future<Output = Result<Option<ExecutionData>, NodeExecutionError>> + Send;

  /// 优化版本的获取所有连接
  fn get_all_connections_optimized(
    &self,
    connection_type: ConnectionKind,
  ) -> impl std::future::Future<Output = Result<Vec<ExecutionData>, NodeExecutionError>> + Send;
}

/// 为NodeExecutionContext实现优化的连接数据获取
impl OptimizedConnectionContext for NodeExecutionContext {
  fn get_connection_data_optimized(
    &self,
    connection_type: ConnectionKind,
    index: usize,
  ) -> impl std::future::Future<Output = Result<Option<ExecutionData>, NodeExecutionError>> + Send {
    async move {
      let manager = get_connection_manager();
      manager.get_connection_data(self, connection_type, index).await
    }
  }

  fn get_all_connections_optimized(
    &self,
    connection_type: ConnectionKind,
  ) -> impl std::future::Future<Output = Result<Vec<ExecutionData>, NodeExecutionError>> + Send {
    async move {
      let manager = get_connection_manager();
      manager.get_all_connections(self, connection_type).await
    }
  }
}
