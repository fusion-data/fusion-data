use std::sync::OnceLock;

use fusion_common::time::now_offset;
use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc},
  filter::{OpValDateTime, OpValInt32},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use hetuflow_core::models::{AgentFilter, AgentForCreate, AgentForUpdate, AgentStatistics, SchedAgent};
use hetuflow_core::{protocol::RegisterAgentRequest, types::AgentStatus};

/// AgentBmc 实现
pub struct AgentBmc;

impl DbBmc for AgentBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
      BmcConfig::new_table("sched_agent")
        .with_id_generated_by_db(false)
        .with_has_created_by(false)
        .with_has_updated_by(false)
        .with_has_updated_at(false)
    })
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

  /// 更新 Agent 心跳时间
  pub async fn update_heartbeat(mm: &ModelManager, agent_id: &str) -> Result<(), SqlError> {
    let update =
      AgentForUpdate { last_heartbeat_at: Some(now_offset()), status: Some(AgentStatus::Online), ..Default::default() };
    Self::update_by_id(mm, agent_id, update).await.map(|_| ())
  }

  /// 更新 Agent 统计信息
  pub async fn update_statistics(
    mm: &ModelManager,
    agent_id: &str,
    statistics: &AgentStatistics,
  ) -> Result<(), SqlError> {
    // 由于 AgentForUpdate 没有 statistics 字段，我们需要直接使用 SQL 更新
    let sql = r#"
      UPDATE sched_agent
      SET statistics = $1
      WHERE id = $2"#;

    let query = sqlx::query(sql).bind(serde_json::to_value(statistics).unwrap()).bind(agent_id);

    mm.dbx().db_postgres()?.execute(query).await?;
    Ok(())
  }

  /// 更新 Agent 任务统计信息
  pub async fn update_task_stats(
    mm: &ModelManager,
    agent_id: &str,
    success: bool,
    response_time_ms: i64,
  ) -> Result<(), SqlError> {
    // 先获取当前统计信息
    let agent = Self::get_by_id(mm, agent_id).await?;
    if let Some(agent) = agent {
      let mut stats = agent.statistics;

      // 更新统计信息
      stats.total_tasks += 1;
      if success {
        stats.success_tasks += 1;
        stats.consecutive_failures = 0;
      } else {
        stats.failure_tasks += 1;
        stats.consecutive_failures += 1;
        stats.last_failure_ms = fusion_common::time::now_epoch_millis();
      }

      // 更新平均响应时间（简单移动平均）
      let total = stats.total_tasks as f64;
      stats.avg_response_ms = (stats.avg_response_ms * (total - 1.0) + response_time_ms as f64) / total;

      // 保存到数据库
      Self::update_statistics(mm, agent_id, &stats).await?;
    }
    Ok(())
  }
}
