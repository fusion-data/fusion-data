use log::{info, warn};
use modelsql::{
  ModelManager,
  filter::{OpValsDateTime, OpValsInt32, OpValsUuid, Page},
  page::PageResult,
};
use ultimate_common::time::now_offset;
use ultimate_core::DataError;
use uuid::Uuid;

use hetuflow_core::{
  models::{
    AgentEntity, AgentFilter, AgentForCreate, AgentForQuery, AgentForUpdate, TaskForUpdate, TaskInstanceFilter,
    TaskInstanceForUpdate,
  },
  types::{AgentStatus, TaskInstanceStatus, TaskStatus},
};

use crate::infra::bmc::{AgentBmc, TaskBmc, TaskInstanceBmc};

pub struct AgentSvc {
  mm: ModelManager,
}

impl AgentSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  /// 创建新的 Agent
  pub async fn create(&self, agent_data: AgentForCreate) -> Result<Uuid, DataError> {
    let id = agent_data.id;
    AgentBmc::insert(&self.mm, agent_data).await?;
    Ok(id)
  }

  /// 根据 ID 获取 Agent
  pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<AgentEntity>, DataError> {
    AgentBmc::get_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  /// 根据 ID 更新 Agent
  pub async fn update_by_id(&self, id: &Uuid, agent_data: AgentForUpdate) -> Result<(), DataError> {
    AgentBmc::update_by_id(&self.mm, id, agent_data).await.map_err(DataError::from)
  }

  /// 根据 ID 删除 Agent
  pub async fn delete_by_id(&self, id: &Uuid) -> Result<(), DataError> {
    AgentBmc::delete_by_id(&self.mm, id).await.map(|_| ()).map_err(DataError::from)
  }

  /// 注册新的 Agent
  pub async fn create_agent(&self, agent_data: AgentForCreate) -> Result<Uuid, DataError> {
    let id = agent_data.id;
    AgentBmc::insert(&self.mm, agent_data).await?;
    Ok(id)
  }

  /// 更新 Agent 状态
  pub async fn update_agent_status(&self, agent_id: &Uuid, status: AgentStatus) -> Result<(), DataError> {
    AgentBmc::update_status(&self.mm, agent_id, status).await.map_err(DataError::from)
  }

  /// 更新 Agent 心跳
  pub async fn update_agent_heartbeat(&self, agent_id: &Uuid) -> Result<(), DataError> {
    AgentBmc::update_heartbeat(&self.mm, agent_id).await.map_err(DataError::from)
  }

  /// 查找在线的 Agent
  pub async fn find_online_agents(&self) -> Result<Vec<AgentEntity>, DataError> {
    AgentBmc::find_online_agents(&self.mm).await.map_err(DataError::from)
  }

  /// 根据 ID 查找 Agent
  pub async fn find_agent_by_id(&self, agent_id: &Uuid) -> Result<AgentEntity, DataError> {
    AgentBmc::find_by_id(&self.mm, agent_id).await.map_err(DataError::from)
  }

  /// 检查离线的 Agent（心跳超时）
  pub async fn check_offline_agents(&self, timeout_seconds: i64) -> Result<Vec<AgentEntity>, DataError> {
    let timeout_time = now_offset() - chrono::Duration::seconds(timeout_seconds);

    let filter = AgentFilter {
      status: Some(OpValsInt32::eq(AgentStatus::Online as i32)),
      last_heartbeat: Some(OpValsDateTime::lt(timeout_time)),
      ..Default::default()
    };

    let offline_agents = AgentBmc::find_many(&self.mm, vec![filter], None).await?;

    // 将超时的 Agent 标记为离线
    for agent in &offline_agents {
      let _ = self.update_agent_status(&agent.id, AgentStatus::Offline).await;
    }

    Ok(offline_agents)
  }

  /// 处理 Agent 离线
  pub async fn handle_agent_offline(&self, agent_id: &Uuid) -> Result<(), DataError> {
    info!("Handling offline agent: {}", agent_id);

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    AgentBmc::update_status(&mm, agent_id, AgentStatus::Offline).await?;

    // TODO: 是否需要强制设置正在运行的任务实例为失败？
    // 获取该 Agent 上运行的任务实例
    let running_instances = TaskInstanceBmc::find_many(
      &mm,
      vec![TaskInstanceFilter {
        agent_id: Some(OpValsUuid::eq(*agent_id)),
        status: Some(OpValsInt32::eq(TaskInstanceStatus::Running as i32)),
        ..Default::default()
      }],
      None,
    )
    .await?;
    for instance in running_instances.iter() {
      warn!("Cancelling task instance {} due to agent {} offline", instance.id, agent_id);

      // 取消任务实例
      let instance_update = TaskInstanceForUpdate {
        status: Some(TaskInstanceStatus::Cancelled),
        completed_at: Some(now_offset()),
        error_message: Some("Agent went offline".to_string()),
        ..Default::default()
      };
      TaskInstanceBmc::update_by_id(&mm, instance.id, instance_update).await?;

      // 重置任务状态，使其可以重新调度
      let task_update = TaskForUpdate { status: Some(TaskStatus::Pending), ..Default::default() };
      TaskBmc::update_by_id(&mm, instance.task_id, task_update).await?;
    }

    mm.dbx().commit_txn().await?;

    if !running_instances.is_empty() {
      info!("Rescheduled {} tasks from offline agent {}", running_instances.len(), agent_id);
    }

    Ok(())
  }

  pub async fn find_many(&self, filter: AgentFilter, page: Option<Page>) -> Result<Vec<AgentEntity>, DataError> {
    AgentBmc::find_many(&self.mm, vec![filter], page).await.map_err(DataError::from)
  }

  pub async fn query(&self, input: AgentForQuery) -> Result<PageResult<AgentEntity>, DataError> {
    AgentBmc::page(&self.mm, vec![input.filter], input.page).await.map_err(DataError::from)
  }
}
