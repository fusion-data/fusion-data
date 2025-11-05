//! Memory Service 组件
//!
//! 提供跨执行共享/持久会话的内存服务抽象与本地内存后端实现。
//! 采用多租户隔离键 (tenant_id:workflow_id:session_id) 管理聊天历史。
//!
//! 约束：
//! - 不引入审计或迁移逻辑（遵循 CLAUDE.md）
//! - 函数级注释，Rust 2024，2 空格缩进

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use hetumind_core::workflow::{Message, NodeExecutionError};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 会话键（多租户隔离）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionKey {
  pub tenant_id: String,
  pub workflow_id: String,
  pub session_id: String,
}

impl SessionKey {
  /// 构造标准化键字符串，用于后端存储
  pub fn as_string(&self) -> String {
    format!("{}:{}:{}", self.tenant_id, self.workflow_id, self.session_id)
  }
}

/// 内存缓冲条目，聚合消息队列与元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMemoryBuffer {
  pub session_id: String,
  pub messages: VecDeque<Message>,
  pub created_at: DateTime<Utc>,
  pub last_updated: DateTime<Utc>,
}

impl WorkflowMemoryBuffer {
  /// 创建新的内存缓冲区
  pub fn new(session_id: String) -> Self {
    let now = Utc::now();
    Self { session_id, messages: VecDeque::new(), created_at: now, last_updated: now }
  }

  /// 追加一条消息到缓冲区
  pub fn add_message(&mut self, message: Message) {
    self.messages.push_back(message);
    self.last_updated = Utc::now();
  }

  /// 获取最近 N 条消息
  pub fn get_recent_messages(&self, count: usize) -> Vec<Message> {
    let len = self.messages.len();
    if len <= count {
      self.messages.iter().cloned().collect()
    } else {
      self.messages.range(len - count..).cloned().collect()
    }
  }

  /// 获取全部消息
  pub fn get_all_messages(&self) -> Vec<Message> {
    self.messages.iter().cloned().collect()
  }

  /// 当前缓冲消息数量
  pub fn len(&self) -> usize {
    self.messages.len()
  }

  /// 判断缓冲是否为空
  pub fn is_empty(&self) -> bool {
    self.messages.is_empty()
  }
}

/// 清理统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
  pub total_entries: usize,
  pub expired_entries: usize,
  pub active_entries: usize,
}

/// 内存服务后端抽象，支持多租户隔离与持久化后端
pub trait MemoryService: Send + Sync {
  /// 获取或创建会话缓冲区
  ///
  /// 参数：tenant_id/workflow_id/session_id 标识唯一会话
  /// 返回：存在则返回已有缓冲，不存在则新建并返回
  fn get_buffer(
    &self,
    tenant_id: &str,
    workflow_id: &str,
    session_id: &str,
  ) -> Result<WorkflowMemoryBuffer, NodeExecutionError>;

  /// 追加存储消息
  ///
  /// 将消息列表追加到指定会话缓冲区末尾，若缓冲不存在则自动创建
  fn store_messages(
    &self,
    tenant_id: &str,
    workflow_id: &str,
    session_id: &str,
    messages: Vec<Message>,
  ) -> Result<(), NodeExecutionError>;

  /// 检索最近 N 条消息
  ///
  /// 当 count 超过缓冲长度时，返回所有消息
  fn retrieve_messages(
    &self,
    tenant_id: &str,
    workflow_id: &str,
    session_id: &str,
    count: usize,
  ) -> Result<Vec<Message>, NodeExecutionError>;

  /// 清理过期会话，返回统计信息
  ///
  /// expired_before 之前未访问的会话将被清理
  fn cleanup(&self, expired_before: DateTime<Utc>) -> Result<CleanupStats, NodeExecutionError>;
}

/// 本地内存后端（带 TTL 清理），适合开发与小租户场景
pub struct InMemoryMemoryService {
  store: Arc<RwLock<HashMap<String, WorkflowMemoryBuffer>>>,
  /// 访问时间缓存（用于过期判断）。这里简化为使用 buffer.last_updated
  default_ttl_seconds: u64,
}

impl InMemoryMemoryService {
  /// 创建新的本地内存后端
  pub fn new(default_ttl_seconds: u64) -> Self {
    Self { store: Arc::new(RwLock::new(HashMap::new())), default_ttl_seconds }
  }

  /// 生成标准 session 键
  fn make_key(&self, tenant_id: &str, workflow_id: &str, session_id: &str) -> String {
    SessionKey {
      tenant_id: tenant_id.to_string(),
      workflow_id: workflow_id.to_string(),
      session_id: session_id.to_string(),
    }
    .as_string()
  }
}

impl MemoryService for InMemoryMemoryService {
  fn get_buffer(
    &self,
    tenant_id: &str,
    workflow_id: &str,
    session_id: &str,
  ) -> Result<WorkflowMemoryBuffer, NodeExecutionError> {
    let key = self.make_key(tenant_id, workflow_id, session_id);
    let mut store = self.store.blocking_write();
    let entry = store.entry(key).or_insert_with(|| WorkflowMemoryBuffer::new(session_id.to_string()));
    Ok(entry.clone())
  }

  fn store_messages(
    &self,
    tenant_id: &str,
    workflow_id: &str,
    session_id: &str,
    messages: Vec<Message>,
  ) -> Result<(), NodeExecutionError> {
    let key = self.make_key(tenant_id, workflow_id, session_id);
    let mut store = self.store.blocking_write();
    let buffer = store.entry(key).or_insert_with(|| WorkflowMemoryBuffer::new(session_id.to_string()));
    for m in messages {
      buffer.add_message(m);
    }
    Ok(())
  }

  fn retrieve_messages(
    &self,
    tenant_id: &str,
    workflow_id: &str,
    session_id: &str,
    count: usize,
  ) -> Result<Vec<Message>, NodeExecutionError> {
    let key = self.make_key(tenant_id, workflow_id, session_id);
    let store = self.store.blocking_read();
    if let Some(buffer) = store.get(&key) { Ok(buffer.get_recent_messages(count)) } else { Ok(Vec::new()) }
  }

  fn cleanup(&self, expired_before: DateTime<Utc>) -> Result<CleanupStats, NodeExecutionError> {
    let mut store = self.store.blocking_write();
    let total = store.len();
    // 结合默认 TTL 与调用方提供的过期阈值，采用两者中较严格的阈值进行清理
    let ttl_threshold = Utc::now() - chrono::Duration::seconds(self.default_ttl_seconds as i64);
    let effective_threshold = if expired_before > ttl_threshold { expired_before } else { ttl_threshold };
    store.retain(|_, buf| buf.last_updated >= effective_threshold);
    let active = store.len();
    Ok(CleanupStats { total_entries: total, expired_entries: total.saturating_sub(active), active_entries: active })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// 测试：消息存储与检索的基本流程（单会话）
  #[test]
  fn test_store_and_retrieve_messages() {
    let svc = InMemoryMemoryService::new(300);
    let tenant_id = "t1";
    let workflow_id = "w1";
    let session_id = "s1";

    // 存储两条消息
    let m1 = Message { role: "user".to_string(), content: "hello".to_string() };
    let m2 = Message { role: "assistant".to_string(), content: "hi".to_string() };
    svc.store_messages(tenant_id, workflow_id, session_id, vec![m1.clone(), m2.clone()]).unwrap();

    // 检索最近 1 条
    let recent = svc.retrieve_messages(tenant_id, workflow_id, session_id, 1).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "hi");

    // 检索全部
    let all = svc.retrieve_messages(tenant_id, workflow_id, session_id, 99).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].content, "hello");
    assert_eq!(all[1].content, "hi");
  }

  /// 测试：清理过期会话（使用超前时间点以清理所有项）
  #[test]
  fn test_cleanup_expired() {
    let svc = InMemoryMemoryService::new(300);
    let tenant_id = "t2";
    let workflow_id = "w2";
    let session_id = "s2";

    let m = Message { role: "user".to_string(), content: "msg".to_string() };
    svc.store_messages(tenant_id, workflow_id, session_id, vec![m]).unwrap();

    // 使用未来时间点，确保全部过期
    let future = Utc::now() + chrono::TimeDelta::try_seconds(3600).unwrap();
    let stats = svc.cleanup(future).unwrap();
    assert_eq!(stats.expired_entries, 1);
    assert_eq!(stats.active_entries, 0);
  }

  /// 测试：并发存储（多线程写同一会话），最终消息数量累加正确
  #[test]
  fn test_concurrent_store() {
    use std::thread;

    let svc = Arc::new(InMemoryMemoryService::new(300));
    let tenant_id = "t3";
    let workflow_id = "w3";
    let session_id = "s3";

    let mut handles = Vec::new();
    for i in 0..10 {
      let svc_cloned = svc.clone();
      handles.push(thread::spawn(move || {
        let msg = Message { role: "user".to_string(), content: format!("m{}", i) };
        svc_cloned.store_messages(tenant_id, workflow_id, session_id, vec![msg]).unwrap();
      }));
    }

    for h in handles {
      h.join().unwrap();
    }

    let all = svc.retrieve_messages(tenant_id, workflow_id, session_id, 999).unwrap();
    assert_eq!(all.len(), 10);
  }
}
