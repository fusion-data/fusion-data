use std::sync::Arc;

use crate::setting::AgentSetting;

use crate::service::{ConnectionManager, TaskExecutor, TaskScheduler};

#[derive(Clone)]
pub struct AgentApplication {
  pub config: Arc<AgentSetting>,
  pub connection_manager: Arc<ConnectionManager>,
  pub task_scheduler: Arc<TaskScheduler>,
  pub task_executor: Arc<TaskExecutor>,
}
