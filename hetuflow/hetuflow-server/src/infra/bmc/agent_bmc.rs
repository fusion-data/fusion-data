use modelsql::{
  ModelManager, SqlError,
  base::DbBmc,
  filter::{OpValsDateTime, OpValsInt32},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use ultimate_common::time::now_offset;
use uuid::Uuid;

use hetuflow_core::types::AgentStatus;

use hetuflow_core::models::{AgentEntity, AgentFilter, AgentForCreate, AgentForUpdate};

/// AgentBmc 实现
pub struct AgentBmc;

impl DbBmc for AgentBmc {
  const TABLE: &str = "sched_agent";
  const ID_GENERATED_BY_DB: bool = false; // Agent ID 由客户端生成
}

generate_pg_bmc_common!(
  Bmc: AgentBmc,
  Entity: AgentEntity,
  ForUpdate: AgentForUpdate,
  ForInsert: AgentForCreate,
);

generate_pg_bmc_filter!(
  Bmc: AgentBmc,
  Entity: AgentEntity,
  Filter: AgentFilter,
);

impl AgentBmc {
  /// 查找在线的 Agent
  pub async fn find_online_agents(mm: &ModelManager) -> Result<Vec<AgentEntity>, SqlError> {
    let filter = AgentFilter { status: Some(OpValsInt32::eq(AgentStatus::Online as i32)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 更新 Agent 心跳时间
  pub async fn update_heartbeat(mm: &ModelManager, agent_id: &Uuid) -> Result<(), SqlError> {
    let update =
      AgentForUpdate { status: Some(AgentStatus::Online), last_heartbeat: Some(now_offset()), ..Default::default() };

    Self::update_by_id(mm, agent_id, update).await.map(|_| ())
  }

  /// 更新 Agent 状态
  pub async fn update_status(mm: &ModelManager, agent_id: &Uuid, status: AgentStatus) -> Result<(), SqlError> {
    let mut update = AgentForUpdate { status: Some(status), ..Default::default() };
    if status == AgentStatus::Online {
      update.last_heartbeat = Some(now_offset());
    }

    Self::update_by_id(mm, agent_id, update).await.map(|_| ())
  }

  /// 检查离线的 Agent
  pub async fn find_offline_agents(mm: &ModelManager, timeout_seconds: i64) -> Result<Vec<AgentEntity>, SqlError> {
    let cutoff_time = now_offset() - chrono::Duration::seconds(timeout_seconds);

    let filter = AgentFilter {
      last_heartbeat: Some(OpValsDateTime::lt(cutoff_time)),
      status: Some(OpValsInt32::eq(AgentStatus::Offline as i32)),
      ..Default::default()
    };

    Self::find_many(mm, vec![filter], None).await
  }
}
