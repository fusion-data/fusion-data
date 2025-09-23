use std::sync::Arc;
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::DataError;
use fusion_core::application::Application;
use fusion_core::configuration::ConfigRegistry;
use fusion_web::config::WebConfig;
use log::{error, info};
use mea::shutdown::ShutdownRecv;
use modelsql::ModelManager;
use modelsql::filter::{OpValsDateTime, OpValsInt32};
use tokio::time::interval;

use hetuflow_core::models::{AgentFilter, AgentForUpdate, ServerFilter, ServerForRegister, ServerForUpdate};
use hetuflow_core::types::{AgentStatus, ServerStatus};

use crate::infra::bmc::{AgentBmc, DistributedLockBmc, ServerBmc};
use crate::model::DistributedLockIds;
use crate::service::TaskGenerationSvc;
use crate::setting::ServerSetting;

/// 调度器服务
pub struct SchedulerSvc {
  mm: ModelManager,
  server_config: Arc<ServerSetting>,
  task_generation_svc: Arc<TaskGenerationSvc>,
  shutdown_rx: std::sync::Mutex<Option<ShutdownRecv>>,
}

impl SchedulerSvc {
  /// 创建新的调度器服务
  pub fn new(mm: ModelManager, server_config: Arc<ServerSetting>, shutdown_rx: ShutdownRecv) -> Self {
    let task_generation_svc = Arc::new(TaskGenerationSvc::new(mm.clone()));
    Self { mm, server_config, task_generation_svc, shutdown_rx: std::sync::Mutex::new(Some(shutdown_rx)) }
  }

  /// 启动调度器服务
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting Scheduler Server with server_id: {}", &self.server_config.server_id);
    let shutdown_rx = self.shutdown_rx.lock().unwrap().take().unwrap();

    // 注册服务器
    self.register_server().await?;

    // 启动生成 Task 循环
    self.start_task_generation(shutdown_rx.clone()).await?;

    // 启动心跳任务
    self.start_heartbeat_task(shutdown_rx.clone()).await;

    // 启动心跳监控任务
    self.start_heartbeat_monitor_task(shutdown_rx).await;

    info!("Scheduler Service started successfully");
    Ok(())
  }

  /// 启动心跳任务
  async fn start_heartbeat_task(&self, shutdown_rx: ShutdownRecv) {
    let mm = self.mm.clone();
    let server_id = self.server_config.server_id.clone();

    tokio::spawn(async move {
      let mut interval = interval(Duration::from_secs(30));

      loop {
        tokio::select! {
          _ = interval.tick() => {
            // 更新服务器心跳时间
            if let Err(e) = Self::update_server_heartbeat(&mm, &server_id).await {
              error!("Failed to update server heartbeat: {}", e);
            }
          }
          _ = shutdown_rx.is_shutdown() => {
            info!("Server heartbeat shutting down");
            break;
          }
        }
      }
    });
  }

  /// 更新服务器心跳时间
  async fn update_server_heartbeat(mm: &ModelManager, server_id: &str) -> Result<(), DataError> {
    let server_update = ServerForUpdate { status: Some(ServerStatus::Active), ..Default::default() };

    ServerBmc::update_by_id(mm, server_id, server_update).await?;
    Ok(())
  }

  /// 启动心跳监控任务
  async fn start_heartbeat_monitor_task(&self, shutdown_rx: ShutdownRecv) {
    let mm = self.mm.clone();
    let agent_overdue_ttl = self.server_config.agent_overdue_ttl;
    let server_overdue_ttl = self.server_config.server_overdue_ttl;

    tokio::spawn(async move {
      let mut interval = interval(Duration::from_secs(60)); // 每分钟检查一次

      loop {
        tokio::select! {
          _ = interval.tick() => {
            // 检查超时的Agent
            if let Err(e) = Self::check_agent_timeouts(&mm, agent_overdue_ttl).await {
              error!("Failed to check agent timeouts: {}", e);
            }

            // 检查超时的Server
            if let Err(e) = Self::check_server_timeouts(&mm, server_overdue_ttl).await {
              error!("Failed to check server timeouts: {}", e);
            }
          }
          _ = shutdown_rx.is_shutdown() => {
            info!("Heartbeat monitor shutting down");
            break;
          }
        }
      }
    });
  }

  /// 检查Agent心跳超时
  async fn check_agent_timeouts(mm: &ModelManager, timeout: Duration) -> Result<(), DataError> {
    let timeout_threshold = now_offset() - timeout;

    // 查找心跳超时的在线Agent
    let filter = AgentFilter {
      status: Some(OpValsInt32::eq(AgentStatus::Online as i32)),
      last_heartbeat: Some(OpValsDateTime::lt(timeout_threshold)),
      ..Default::default()
    };

    let timeout_agents = AgentBmc::find_many(mm, vec![filter], None).await?;

    for agent in timeout_agents {
      info!("Agent {} heartbeat timeout, marking as offline", agent.id);

      let update = AgentForUpdate { status: Some(AgentStatus::Offline), ..Default::default() };

      AgentBmc::update_by_id(mm, agent.id, update).await?;
    }

    Ok(())
  }

  /// 检查Server心跳超时
  async fn check_server_timeouts(mm: &ModelManager, timeout: Duration) -> Result<(), DataError> {
    let timeout_threshold = now_offset() - timeout;

    // 查找心跳超时的活跃Server
    let filter = ServerFilter {
      status: Some(OpValsInt32::eq(ServerStatus::Active as i32)),
      updated_at: Some(OpValsDateTime::lt(timeout_threshold)),
      ..Default::default()
    };

    let timeout_servers = ServerBmc::find_many(mm, vec![filter], None).await?;

    for server in timeout_servers {
      info!("Server {} heartbeat timeout, marking as inactive", server.id);

      let update = ServerForUpdate { status: Some(ServerStatus::Inactive), ..Default::default() };

      ServerBmc::update_by_id(mm, server.id, update).await?;
    }

    Ok(())
  }

  /// 注册服务器
  async fn register_server(&self) -> Result<(), DataError> {
    info!("Registering server: {}", &self.server_config.server_id);

    let web_config: WebConfig = Application::global().get_config()?;

    let server = ServerForRegister {
      id: self.server_config.server_id.clone(),
      name: self.server_config.server_name.clone(),
      address: web_config.server_addr.clone(),
      status: ServerStatus::Active,
    };
    ServerBmc::register(&self.mm, server).await?;
    info!("Server {} registered", &self.server_config.server_id);
    Ok(())
  }

  async fn start_task_generation(&self, shutdown_rx: ShutdownRecv) -> Result<(), DataError> {
    let task_generation_svc = self.task_generation_svc.clone();
    let mut interval = interval(self.server_config.job_check_interval);
    let duration = self.server_config.job_check_duration;
    tokio::spawn(async move {
      loop {
        tokio::select! {
            _ = interval.tick() => {
                let from_time = now_offset();
                let to_time = from_time + duration;

                // 生成定时任务
                if let Err(e) = task_generation_svc.generate_tasks_for_schedule(from_time, to_time).await {
                    error!("Task generation failed: {}", e);
                }

                // 生成重试任务
                if let Err(e) = task_generation_svc.generate_retry_tasks().await {
                    error!("Retry task generation failed: {}", e);
                }
            }
            _ = shutdown_rx.is_shutdown() => {
                info!("Task generation shutting down");
                break;
            }
        }
      }
    });

    Ok(())
  }

  /// 释放领导权
  pub async fn release_leadership(&self) -> Result<(), DataError> {
    DistributedLockBmc::release_leadership(
      &self.mm,
      DistributedLockIds::SCHED_SERVER_LEADER,
      &self.server_config.server_id,
    )
    .await?;
    info!("Released leadership for server: {}", self.server_config.server_id);
    Ok(())
  }
}
