use fusion_common::time::now_offset;
use fusionsql::{
  ModelManager, SqlError,
  base::DbBmc,
  filter::{OpValDateTime, OpValInt32},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use hetuflow_core::models::{AgentFilter, AgentForCreate, AgentForUpdate, SchedAgent};
use hetuflow_core::{protocol::RegisterAgentRequest, types::AgentStatus};

/// AgentBmc 实现
pub struct AgentBmc;

impl DbBmc for AgentBmc {
  const TABLE: &str = "sched_agent";
  const ID_GENERATED_BY_DB: bool = false;
  fn _has_created_by() -> bool {
    false
  }
  fn _has_updated_at() -> bool {
    false
  }
  fn _has_updated_by() -> bool {
    false
  }
}

generate_pg_bmc_common!(
  Bmc: AgentBmc,
  Entity: SchedAgent,
  ForUpdate: AgentForUpdate,
  ForInsert: AgentForCreate,
);

generate_pg_bmc_filter!(
  Bmc: AgentBmc,
  Entity: SchedAgent,
  Filter: AgentFilter,
);

impl AgentBmc {
  /// 查找在线的 Agent
  pub async fn find_online_agents(mm: &ModelManager) -> Result<Vec<SchedAgent>, SqlError> {
    let filter = AgentFilter { status: Some(OpValInt32::eq(AgentStatus::Online as i32)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 更新 Agent 状态
  pub async fn update_status(mm: &ModelManager, agent_id: &str, status: AgentStatus) -> Result<(), SqlError> {
    let mut update = AgentForUpdate { status: Some(status), ..Default::default() };
    if status == AgentStatus::Online {
      update.last_heartbeat_at = Some(now_offset());
    }

    Self::update_by_id(mm, agent_id, update).await.map(|_| ())
  }

  /// 检查离线的 Agent
  pub async fn find_offline_agents(mm: &ModelManager, timeout_seconds: i64) -> Result<Vec<SchedAgent>, SqlError> {
    let cutoff_time = now_offset() - chrono::Duration::seconds(timeout_seconds);

    let filter = AgentFilter {
      last_heartbeat_at: Some(OpValDateTime::lt(cutoff_time)),
      status: Some(OpValInt32::eq(AgentStatus::Offline as i32)),
      ..Default::default()
    };

    Self::find_many(mm, vec![filter], None).await
  }

  pub async fn register(
    mm: &ModelManager,
    agent_id: &str,
    payload: &RegisterAgentRequest,
  ) -> Result<SchedAgent, SqlError> {
    let sql = r#"
      insert into sched_agent (id, address, status, capabilities, last_heartbeat_at)
      values ($1, $2, $3, $4, $5)
      on conflict (id) do update set
        address = excluded.address,
        status = excluded.status,
        capabilities = excluded.capabilities,
        last_heartbeat_at = excluded.last_heartbeat_at
      returning *"#;
    let query = sqlx::query_as(sql)
      .bind(agent_id)
      .bind(&payload.address)
      .bind(AgentStatus::Online)
      .bind(&payload.capabilities)
      .bind(now_offset());

    let agent = mm.dbx().db_postgres()?.fetch_one::<SchedAgent, _>(query).await?;
    Ok(agent)
  }
}
