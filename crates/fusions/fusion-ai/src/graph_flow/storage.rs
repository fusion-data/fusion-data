use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::graph_flow::{Context, error::Result, graph::Graph};

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
  pub id: String,
  pub graph_id: String,
  pub current_task_id: String,
  /// Optional status message from the last executed task
  pub status_message: Option<String>,
  pub context: crate::graph_flow::context::Context,
}

impl Session {
  pub fn new_from_task(sid: String, task_name: &str) -> Self {
    Self {
      id: sid,
      graph_id: "default".to_string(),
      current_task_id: task_name.to_string(),
      status_message: None,
      context: Context::new(),
    }
  }
}

/// Trait for storing and retrieving graphs
#[async_trait]
pub trait GraphStorage: Send + Sync {
  async fn save(&self, id: String, graph: Arc<Graph>) -> Result<()>;
  async fn get(&self, id: &str) -> Result<Option<Arc<Graph>>>;
  async fn delete(&self, id: &str) -> Result<()>;
}

/// Trait for storing and retrieving sessions
#[async_trait]
pub trait SessionStorage: Send + Sync {
  async fn save(&self, session: Session) -> Result<()>;
  async fn get(&self, id: &str) -> Result<Option<Session>>;
  async fn delete(&self, id: &str) -> Result<()>;
}

/// In-memory implementation of GraphStorage
pub struct InMemoryGraphStorage {
  graphs: Arc<DashMap<String, Arc<Graph>>>,
}

impl Default for InMemoryGraphStorage {
  fn default() -> Self {
    Self::new()
  }
}

impl InMemoryGraphStorage {
  pub fn new() -> Self {
    Self { graphs: Arc::new(DashMap::new()) }
  }
}

#[async_trait]
impl GraphStorage for InMemoryGraphStorage {
  async fn save(&self, id: String, graph: Arc<Graph>) -> Result<()> {
    self.graphs.insert(id, graph);
    Ok(())
  }

  async fn get(&self, id: &str) -> Result<Option<Arc<Graph>>> {
    Ok(self.graphs.get(id).map(|entry| entry.clone()))
  }

  async fn delete(&self, id: &str) -> Result<()> {
    self.graphs.remove(id);
    Ok(())
  }
}

/// In-memory implementation of SessionStorage
pub struct InMemorySessionStorage {
  sessions: Arc<DashMap<String, Session>>,
}

impl Default for InMemorySessionStorage {
  fn default() -> Self {
    Self::new()
  }
}

impl InMemorySessionStorage {
  pub fn new() -> Self {
    Self { sessions: Arc::new(DashMap::new()) }
  }
}

#[async_trait]
impl SessionStorage for InMemorySessionStorage {
  async fn save(&self, session: Session) -> Result<()> {
    self.sessions.insert(session.id.clone(), session);
    Ok(())
  }

  async fn get(&self, id: &str) -> Result<Option<Session>> {
    Ok(self.sessions.get(id).map(|entry| entry.clone()))
  }

  async fn delete(&self, id: &str) -> Result<()> {
    self.sessions.remove(id);
    Ok(())
  }
}
