use std::{sync::Arc, time::Duration};

use fusion_common::time::now_offset;
use fusion_core::DataError;
use fusionsql::{
  ModelManager,
  filter::{OpValDateTime, OpValInt32},
};
use hetuflow_core::{
  models::{AgentFilter, AgentForUpdate, ServerFilter, ServerForUpdate},
  types::{AgentStatus, ServerStatus},
};
use log::{error, info};
use mea::shutdown::ShutdownRecv;

use crate::{
  infra::bmc::{AgentBmc, DistributedLockBmc, ServerBmc},
  model::DistributedLockIds,
  setting::HetuflowSetting,
};

use super::{IsLeader, LoadBalancer};

pub struct LeaderOrFollowerRunner {
  setting: Arc<HetuflowSetting>,
  mm: ModelManager,
  shutdown_rx: ShutdownRecv,
  is_leader: IsLeader,
  load_balancer: LoadBalancer,
}

impl LeaderOrFollowerRunner {
  pub(super) fn new(
    setting: Arc<HetuflowSetting>,
    mm: ModelManager,
    shutdown_rx: ShutdownRecv,
    is_leader: IsLeader,
  ) -> Self {
    let load_balancer = LoadBalancer::new(mm.clone());
    Self { setting, mm, shutdown_rx, is_leader, load_balancer }
  }

  pub(super) async fn run_loop(&self) {
    let (ttl, token_increment, heartbeat_interval) = DistributedLockBmc::get_recommended_config();
    let mut interval = tokio::time::interval(heartbeat_interval);
    loop {
      tokio::select! {
        _ = interval.tick() => {
          self.try_acquire_leader_or_follower(&ttl, &token_increment).await;
          if self.is_leader.leader() {
            self.perform_leader_operations().await;
          }
        }
        _ = self.shutdown_rx.is_shutdown() => {
          info!("Shutdown signal received, stopping BrokerRunner loop.");
          break;
        }
      }
    }
  }

  async fn perform_leader_operations(&self) {
    if let Err(e) = self.load_balancer.rebalance_if_needed().await {
      error!("Failed try to rebalance, error: {:?}", e);
    }
    if let Err(e) = self.check_agent_timeouts().await {
      error!("Failed to check agent timeouts, error: {:?}", e);
    }
    if let Err(e) = self.check_server_timeouts().await {
      error!("Failed to check server timeouts, error: {:?}", e);
    }
  }

  /// 检查Agent心跳超时
  async fn check_agent_timeouts(&self) -> Result<(), DataError> {
    let timeout_threshold = now_offset() - self.setting.server.agent_overdue_ttl;
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 查找心跳超时的在线Agent
    let filter = AgentFilter {
      status: Some(OpValInt32::eq(AgentStatus::Online as i32)),
      last_heartbeat_at: Some(OpValDateTime::lt(timeout_threshold)),
      ..Default::default()
    };
    let timeout_agents = AgentBmc::find_many(&mm, vec![filter], None).await?;

    for agent in timeout_agents {
      info!("Agent {} heartbeat timeout, marking as offline", agent.id);
      let update = AgentForUpdate { status: Some(AgentStatus::Offline), ..Default::default() };
      AgentBmc::update_by_id(&mm, agent.id, update).await?;
    }

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  /// 检查Server心跳超时
  async fn check_server_timeouts(&self) -> Result<(), DataError> {
    let timeout_threshold = now_offset() - self.setting.server.server_heartbeat_ttl;
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 查找心跳超时的在线Server
    let filter = ServerFilter {
      status: Some(OpValInt32::eq(ServerStatus::Active as i32)),
      last_heartbeat_at: Some(OpValDateTime::lt(timeout_threshold)),
      ..Default::default()
    };
    let timeout_servers = ServerBmc::find_many(&mm, vec![filter], None).await?;

    for server in timeout_servers {
      info!("Server {} heartbeat timeout, marking as inactive", server.id);

      let update = ServerForUpdate { status: Some(ServerStatus::Inactive), ..Default::default() };

      ServerBmc::update_by_id(&mm, server.id, update).await?;
    }

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  async fn try_acquire_leader_or_follower(&self, ttl: &Duration, token_increment: &Duration) {
    let mm = self.mm.get_txn_clone();
    if let Err(e) = mm.dbx().begin_txn().await {
      error!("Failed to begin try_acquire_leader_or_follower transaction: {:?}", e);
      return;
    }

    let server_id = self.setting.server.server_id.as_str();

    let ret = DistributedLockBmc::try_acquire_or_update(
      &mm,
      DistributedLockIds::SCHED_SERVER_LEADER,
      server_id,
      ttl,
      token_increment,
    )
    .await;
    match ret {
      Ok(Some(_)) => {
        self.is_leader.set_leader();
      }
      Ok(None) => {
        self.is_leader.set_follower();
      }
      Err(e) => {
        error!("Leadership acquisition error: {:?}", e);
      }
    }

    // 更新服务器心跳时间
    let server_update = ServerForUpdate {
      status: Some(ServerStatus::Active),
      last_heartbeat_at: Some(now_offset()),
      ..Default::default()
    };
    if let Err(e) = ServerBmc::update_by_id(&mm, server_id, server_update).await {
      error!("Failed to update server heartbeat: {:?}", e);
    }

    if let Err(e) = mm.dbx().commit_txn().await {
      error!("Failed to commit try_acquire_leader_or_follower transaction: {:?}", e);
    }
  }
}
