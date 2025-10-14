use std::sync::Arc;

use fusion_core::{DataError, application::Application, configuration::ConfigRegistry};
use fusion_web::config::WebConfig;
use fusionsql::ModelManager;
use hetuflow_core::{models::ServerForRegister, types::ServerStatus};
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use tokio::task::JoinHandle;

use crate::{
  infra::bmc::{DistributedLockBmc, ServerBmc},
  model::DistributedLockIds,
  setting::HetuflowSetting,
};

use super::{IsLeader, LeaderOrFollowerRunner};

#[derive(Clone)]
pub struct Broker {
  setting: Arc<HetuflowSetting>,
  mm: ModelManager,
  is_leader: IsLeader,
}

impl Broker {
  pub fn new(setting: Arc<HetuflowSetting>, mm: ModelManager) -> Self {
    Self { setting, mm, is_leader: IsLeader::default() }
  }

  pub async fn start(&self, shutdown_rx: ShutdownRecv) -> Result<(), DataError> {
    self.register_server().await?;
    self.start_leader_and_follower_loop(shutdown_rx.clone()).await?;

    let is_leader = self.is_leader.clone();
    let mm = self.mm.clone();
    let setting = self.setting.clone();
    tokio::spawn(async move {
      shutdown_rx.is_shutdown().await;

      // 释放领导者身份
      if is_leader.leader()
        && let Err(e) = Self::release_leadership(&mm, &setting.server.server_id).await
      {
        error!("Failed to release leadership: {:?}", e);
      }
    });

    Ok(())
  }

  pub fn is_leader(&self) -> bool {
    self.is_leader.leader()
  }

  /// 启动获取领导者身份（或刷新领导者身份过期时间）循环
  ///
  /// 该循环会尝试获取领导者身份，如果获取失败，则会等待 heartbeat_interval 后重试。
  /// 如果获取成功，则会将 `is_leader` 设置为 `true`，否则设置为 `false`。
  async fn start_leader_and_follower_loop(&self, shutdown_rx: ShutdownRecv) -> Result<JoinHandle<()>, DataError> {
    // 启动领导者选举监控
    let is_leader = self.is_leader.clone();

    let broker_runner = LeaderOrFollowerRunner::new(self.setting.clone(), self.mm.clone(), shutdown_rx, is_leader);
    let handle = tokio::spawn(async move { broker_runner.run_loop().await });
    Ok(handle)
  }

  /// 注册服务器
  async fn register_server(&self) -> Result<(), DataError> {
    info!("Registering server: {}", &self.setting.server.server_id);

    let web_config: WebConfig = Application::global().get_config()?;

    let server = ServerForRegister {
      id: self.setting.server.server_id.clone(),
      name: self.setting.server.server_name.clone(),
      address: web_config.server_addr.clone(),
      status: ServerStatus::Active,
    };
    ServerBmc::register(&self.mm, server).await?;
    info!("Server {} registered", &self.setting.server.server_id);
    Ok(())
  }

  /// 释放领导权
  async fn release_leadership(mm: &ModelManager, server_id: &str) -> Result<(), DataError> {
    DistributedLockBmc::release_leadership(mm, DistributedLockIds::SCHED_SERVER_LEADER, server_id).await?;
    info!("Released leadership for server: {}", server_id);
    Ok(())
  }
}
