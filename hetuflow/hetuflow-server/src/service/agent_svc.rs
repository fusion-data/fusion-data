use fusion_common::page::Page;
use fusion_common::time::{now_epoch_millis, now_offset};
use fusion_core::DataError;
use fusionsql::{
  ModelManager,
  filter::{OpValDateTime, OpValInt32, OpValString},
  page::PageResult,
};
use log::{info, warn};

use hetuflow_core::{
  models::{
    AgentFilter, AgentForCreate, AgentForQuery, AgentForUpdate, SchedAgent, TaskForUpdate, TaskInstanceFilter,
    TaskInstanceForUpdate,
  },
  protocol::{AgentRegisterResponse, RegisterAgentRequest},
  types::{AgentStatus, TaskInstanceStatus, TaskStatus},
};

use crate::{
  infra::bmc::{AgentBmc, TaskBmc, TaskInstanceBmc},
  service::{JweError, JweSvc},
  setting::HetuflowSetting,
};

pub struct AgentSvc {
  mm: ModelManager,
  jwe_service: Option<JweSvc>,
}

impl AgentSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm, jwe_service: None }
  }

  /// 创建带有JWE配置的AgentSvc
  pub fn new_with_setting(mm: ModelManager, setting: &HetuflowSetting) -> Result<Self, DataError> {
    let jwe_service = if let Some(jwe_config) = &setting.jwe { Some(JweSvc::new(jwe_config.clone())?) } else { None };
    Ok(Self { mm, jwe_service })
  }

  /// 创建新的 Agent
  pub async fn create(&self, agent_data: AgentForCreate) -> Result<String, DataError> {
    let id = agent_data.id.clone();
    AgentBmc::insert(&self.mm, agent_data).await?;
    Ok(id)
  }

  /// 根据 ID 获取 Agent
  pub async fn get_by_id(&self, id: &str) -> Result<Option<SchedAgent>, DataError> {
    AgentBmc::get_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  /// 根据 ID 更新 Agent
  pub async fn update_by_id(&self, id: &str, agent_data: AgentForUpdate) -> Result<(), DataError> {
    AgentBmc::update_by_id(&self.mm, id, agent_data).await.map_err(DataError::from)
  }

  /// 根据 ID 删除 Agent
  pub async fn delete_by_id(&self, id: &str) -> Result<(), DataError> {
    AgentBmc::delete_by_id(&self.mm, id).await.map(|_| ()).map_err(DataError::from)
  }

  /// 注册新的 Agent
  pub async fn create_agent(&self, agent_data: AgentForCreate) -> Result<String, DataError> {
    let id = agent_data.id.clone();
    AgentBmc::insert(&self.mm, agent_data).await?;
    Ok(id)
  }

  /// 更新 Agent 状态
  pub async fn update_agent_status(&self, agent_id: &str, status: AgentStatus) -> Result<(), DataError> {
    AgentBmc::update_status(&self.mm, agent_id, status).await.map_err(DataError::from)
  }

  /// 更新 Agent 心跳
  pub async fn update_agent_heartbeat(&self, agent_id: &str) -> Result<(), DataError> {
    let update =
      AgentForUpdate { status: Some(AgentStatus::Online), last_heartbeat_at: Some(now_offset()), ..Default::default() };
    AgentBmc::update_by_id(&self.mm, agent_id, update).await.map_err(DataError::from)
  }

  /// 查找在线的 Agent
  pub async fn find_online_agents(&self) -> Result<Vec<SchedAgent>, DataError> {
    AgentBmc::find_online_agents(&self.mm).await.map_err(DataError::from)
  }

  /// 根据 ID 查找 Agent
  pub async fn find_by_id(&self, agent_id: &str) -> Result<SchedAgent, DataError> {
    AgentBmc::find_by_id(&self.mm, agent_id).await.map_err(DataError::from)
  }

  /// 检查离线的 Agent（心跳超时）
  pub async fn check_offline_agents(&self, timeout_seconds: i64) -> Result<Vec<SchedAgent>, DataError> {
    let timeout_time = now_offset() - chrono::Duration::seconds(timeout_seconds);

    let filter = AgentFilter {
      status: Some(OpValInt32::eq(AgentStatus::Online as i32)),
      last_heartbeat_at: Some(OpValDateTime::lt(timeout_time)),
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
  pub async fn handle_agent_offline(&self, agent_id: &str) -> Result<(), DataError> {
    info!("Handling offline agent: {}", agent_id);

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    AgentBmc::update_status(&mm, agent_id, AgentStatus::Offline).await?;

    // TODO: 是否需要强制设置正在运行的任务实例为失败？
    // 获取该 Agent 上运行的任务实例
    let running_instances = TaskInstanceBmc::find_many(
      &mm,
      vec![TaskInstanceFilter {
        agent_id: Some(OpValString::eq(agent_id)),
        status: Some(OpValInt32::eq(TaskInstanceStatus::Running as i32)),
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

  pub async fn find_many(&self, filter: AgentFilter, page: Option<Page>) -> Result<Vec<SchedAgent>, DataError> {
    AgentBmc::find_many(&self.mm, vec![filter], page).await.map_err(DataError::from)
  }

  pub async fn query(&self, input: AgentForQuery) -> Result<PageResult<SchedAgent>, DataError> {
    AgentBmc::page(&self.mm, vec![input.filter], input.page).await.map_err(DataError::from)
  }

  pub async fn handle_register(
    &self,
    agent_id: &str,
    payload: &RegisterAgentRequest,
  ) -> Result<AgentRegisterResponse, DataError> {
    // JWE双重验证逻辑
    if let Some(jwe_service) = &self.jwe_service {
      // 如果配置了JWE服务，则必须提供有效的JWE Token
      let jwe_token = payload
        .jwe_token
        .as_ref()
        .ok_or_else(|| DataError::bad_request("JWE Token is required for agent registration"))?;

      // 验证JWE Token
      match jwe_service.verify_token(jwe_token, agent_id.to_string()) {
        Ok(token_payload) => {
          info!("Agent {} JWE token verified successfully, server_id: {}", agent_id, token_payload.server_id);
        }
        Err(JweError::TokenExpired) => {
          warn!("Agent registration failed: JWE token expired, agent_id: {}", agent_id);
          return Ok(AgentRegisterResponse {
            success: false,
            message: "JWE token has expired, please generate a new token".to_string(),
            agent: None,
            server_time: now_epoch_millis(),
          });
        }
        Err(JweError::AgentIdMismatch { expected, actual }) => {
          warn!(
            "Agent registration failed: Agent ID: {} mismatch (expected: {}, actual: {})",
            agent_id, expected, actual
          );
          return Ok(AgentRegisterResponse {
            success: false,
            message: "Agent ID mismatch in JWE token".to_string(),
            agent: None,
            server_time: now_epoch_millis(),
          });
        }
        Err(e) => {
          warn!("Agent registration failed: JWE token verification agent_Id: {}, error: {:?}", agent_id, e);
          return Ok(AgentRegisterResponse {
            success: false,
            message: format!("JWE token verification failed: {}", e),
            agent: None,
            server_time: now_epoch_millis(),
          });
        }
      }
    } else {
      // 如果未配置JWE服务，记录警告但允许注册（向后兼容）
      if payload.jwe_token.is_some() {
        warn!("Agent {} provided JWE token but JWE service is not configured", agent_id);
      }
      info!("Agent {} registration without JWE verification (JWE service not configured)", agent_id);
    }

    // 执行Agent注册
    let agent = AgentBmc::register(&self.mm, agent_id, payload).await?;
    let response = AgentRegisterResponse {
      success: true,
      message: "Agent registered successfully".to_string(),
      agent: Some(agent),
      server_time: now_epoch_millis(),
    };
    Ok(response)
  }
}
