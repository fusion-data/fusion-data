use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use fusion_core::{DataError, application::Application};
use fusion_db::DbPlugin;
use log::{debug, error, info};
use modelsql::{ModelManager, store::DbxError};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::service::{AgentManager, LoadBalancer, LogReceiver, SchedulerSvc};
use crate::{
  gateway::{ConnectionManager, GatewaySvc, MessageHandler},
  infra::bmc::DistributedLockBmc,
  model::DistributedLockIds,
};
use crate::{
  model::{GatewayCommandRequest, HealthStatus},
  setting::HetuflowSetting,
};

/// Hetuflow 应用容器
#[derive(Clone)]
pub struct ServerApplication {
  pub setting: Arc<HetuflowSetting>,
  pub is_leader: Arc<AtomicBool>,
  shutdown_tx: broadcast::Sender<()>,
  pub mm: ModelManager,
  pub scheduler_svc: Arc<SchedulerSvc>,
  pub gateway_svc: Arc<GatewaySvc>,
  agent_manager: Arc<AgentManager>,
  pub connection_manager: Arc<ConnectionManager>,
  pub message_handler: Arc<MessageHandler>,
  load_balancer: Arc<LoadBalancer>,
  gateway_command_tx: mpsc::UnboundedSender<GatewayCommandRequest>,
  log_receiver: Arc<tokio::sync::Mutex<LogReceiver>>,
}

impl ServerApplication {
  pub async fn new() -> Result<Self, DataError> {
    Self::new_with_source::<config::Environment>(None).await
  }

  pub async fn new_with_source<S>(config_source: Option<S>) -> Result<Self, DataError>
  where
    S: config::Source + Send + Sync + 'static,
  {
    // 构建底层 Application 与插件
    let application = Application::builder().add_plugin(DbPlugin).build().await?;
    if let Some(config_source) = config_source {
      let config_registry = application.config_registry();
      config_registry.add_config_source(config_source)?;
      config_registry.reload()?;
    }

    let config = Arc::new(HetuflowSetting::load(application.config_registry())?);

    // 获取 ModelManager
    let mm = application.get_component::<ModelManager>()?;

    // 创建关闭信号通道
    let (shutdown_tx, _) = broadcast::channel(1);

    // 创建通信通道
    let (gateway_command_tx, gateway_command_rx) = mpsc::unbounded_channel();

    // 创建网关组件
    let connection_manager = Arc::new(ConnectionManager::new());
    let message_handler = Arc::new(MessageHandler::new(connection_manager.clone()));

    // 初始化核心组件
    let scheduler_svc = Arc::new(SchedulerSvc::new(mm.clone(), Arc::new(config.server.clone()), shutdown_tx.clone()));

    // 将 ConnectionManager 作为 AgentRegistry 传递给 AgentManager
    let agent_manager = Arc::new(AgentManager::new(mm.clone(), connection_manager.clone(), config.clone()));

    let gateway_svc =
      Arc::new(GatewaySvc::new(connection_manager.clone(), message_handler.clone(), gateway_command_rx));

    let is_leader = Arc::new(AtomicBool::new(false));
    let load_balancer = Arc::new(LoadBalancer::new(mm.clone(), config.server.server_id.clone()));

    // 创建日志接收器
    let mut log_receiver = LogReceiver::new(Arc::new(config.task_log.clone()));
    log_receiver.set_gateway_sender(gateway_command_tx.clone());
    let log_receiver = Arc::new(tokio::sync::Mutex::new(log_receiver));

    Ok(Self {
      setting: config,
      is_leader,
      shutdown_tx,
      mm,
      scheduler_svc,
      gateway_svc,
      agent_manager,
      connection_manager,
      message_handler,
      load_balancer,
      gateway_command_tx,
      log_receiver,
    })
  }

  /// 启动调度器应用
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting Hetuflow Application");

    // 1. 尝试获取领导者身份
    self.start_leader_and_follower_loop().await?;

    // 2. 启动通用服务 (所有实例都启动)
    self.start_common_services().await?;

    // 3. 启动健康检查
    self.start_health_checks().await?;

    info!("Hetuflow Application started successfully");
    Ok(())
  }

  /// 启动获取领导者身份（或刷新领导者身份过期时间）循环
  ///
  /// 该循环会尝试获取领导者身份，如果获取失败，则会等待 heartbeat_interval 后重试。
  /// 如果获取成功，则会将 `is_leader` 设置为 `true`，否则设置为 `false`。
  async fn start_leader_and_follower_loop(&self) -> Result<(), DataError> {
    info!("Starting follower mode");

    // 启动领导者选举监控
    let is_leader = self.is_leader.clone();
    let mm = self.mm.clone();
    let server_id = self.setting.server.server_id.to_string();
    let mut shutdown_rx = self.shutdown_tx.subscribe();
    let load_balancer = self.load_balancer.clone();
    tokio::spawn(async move {
      let (ttl, token_increment_interval, heartbeat_interval) = DistributedLockBmc::get_recommended_config();
      let mut interval = tokio::time::interval(heartbeat_interval);
      loop {
        tokio::select! {
          _ = interval.tick() => {
            _try_acquire_or_heartbeat_leadership(&mm, &server_id, &is_leader, &load_balancer, &ttl, &token_increment_interval).await;
          }
          _ = shutdown_rx.recv() => {
            info!("Shutdown signal received, stopping leader and follower loop.");
            break;
          }
        }
      }
    });

    Ok(())
  }

  /// 启动通用服务
  async fn start_common_services(&self) -> Result<(), DataError> {
    info!("Starting common services");

    // 启用 Scheduler 服务
    self.scheduler_svc.start().await?;

    // 启动 Agent 管理器（事件订阅）
    let agent_manager = self.agent_manager.clone();
    if let Err(e) = agent_manager.start().await {
      error!("AgentManager start error: {:?}", e);
    }

    // 启动网关服务
    let gateway_svc = self.gateway_svc.clone();
    if let Err(e) = gateway_svc.start().await {
      error!("Gateway service error: {:?}", e);
    }

    // 启动日志接收器
    let log_receiver = self.log_receiver.clone();
    tokio::spawn(async move {
      let mut receiver = log_receiver.lock().await;
      if let Err(e) = receiver.start().await {
        error!("LogReceiver start error: {:?}", e);
      }
    });

    // 启动 HTTP API 服务 (/api/v1)
    let app_state = self.clone();
    tokio::spawn(async move {
      let router = crate::endpoint::routes().with_state(app_state);
      if let Err(e) = fusion_web::server::init_server(router).await {
        error!("HTTP server error: {:?}", e);
      }
    });

    Ok(())
  }

  /// 启动健康检查
  async fn start_health_checks(&self) -> Result<(), DataError> {
    // 数据库健康检查
    let mm = self.mm.clone();
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
      loop {
        interval.tick().await;
        match Self::check_database_health(&mm).await {
          Ok(_) => debug!("Database health check passed"),
          Err(e) => error!("Database health check failed: {:?}", e),
        }
      }
    });

    // Agent 心跳超时清理
    let connection_manager = self.connection_manager.clone();
    let agent_heartbeat_ttl = self.setting.server.agent_overdue_ttl;
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
      loop {
        interval.tick().await;
        if let Err(e) = connection_manager.cleanup_stale_connections(agent_heartbeat_ttl).await {
          error!("Connection cleanup failed: {:?}", e);
        }
      }
    });

    // Agent 健康检查与僵尸任务清理
    let agent_manager = self.agent_manager.clone();
    tokio::spawn(async move {
      let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(120));
      loop {
        interval.tick().await;
        if let Err(e) = agent_manager.check_agent_health().await {
          error!("Agent health check failed: {:?}", e);
        }
      }
    });

    Ok(())
  }

  /// 数据库健康检查
  async fn check_database_health(mm: &ModelManager) -> Result<(), DataError> {
    mm.dbx()
      .use_postgres(|dbx| async move {
        sqlx::query("SELECT 1").fetch_one(dbx.db()).await.map_err(DbxError::from)?;
        Ok(())
      })
      .await
      .map_err(|e| DataError::server_error(format!("Database health check failed: {}", e)))
  }

  /// 优雅关闭
  pub async fn shutdown(&self) -> Result<(), DataError> {
    info!("Shutting down Hetuflow Application");

    // 释放领导者身份
    if self.is_leader.load(Ordering::Relaxed)
      && let Err(e) = self.scheduler_svc.release_leadership().await
    {
      error!("Failed to release leadership: {:?}", e);
    }

    // 发送关闭信号
    self.shutdown_tx.send(()).unwrap();

    // 停止各种服务
    // TODO: 各服务的关闭是否有先后顺序？
    self.gateway_svc.stop().await?;
    // self.connection_manager.stop().await?;
    // self.message_handler.stop().await?;
    // self.agent_manager.stop().await?;

    info!("Hetuflow Application shutdown complete");
    Ok(())
  }

  pub fn is_leader(&self) -> bool {
    self.is_leader.load(Ordering::Relaxed)
  }

  pub async fn health_status(&self) -> Result<HealthStatus, DataError> {
    let db = self.mm.dbx().db_postgres()?;
    let db_size = db.db().size();

    let agent_size = self.connection_manager.get_online_count()?;

    let body = HealthStatus::new(db_size, agent_size);
    Ok(body)
  }

  pub async fn agent_stats(&self) -> Result<serde_json::Value, DataError> {
    self.agent_manager.get_stats().await
  }

  pub async fn send_gateway_command(&self, command: GatewayCommandRequest) -> Result<Uuid, DataError> {
    let message_id = match &command {
      GatewayCommandRequest::Single { command, .. } => command.id(),
      GatewayCommandRequest::Broadcast { command } => command.id(),
    };
    self.gateway_command_tx.send(command)?;
    Ok(message_id)
  }

  /// 获取应用配置
  pub fn setting(&self) -> &HetuflowSetting {
    &self.setting
  }

  /// 处理Agent转发的日志消息
  pub async fn handle_log_message(&self, log_batch: hetuflow_core::protocol::LogBatch) -> Result<(), DataError> {
    let log_receiver = self.log_receiver.clone();
    let receiver = log_receiver.lock().await;
    // 处理日志批次 - 直接调用内部方法
    match receiver.write_log_batch(&log_batch).await {
      Ok(_) => Ok(()),
      Err(e) => {
        log::error!("处理日志批次失败: {}", e);
        Err(e)
      }
    }
  }
}

async fn _try_acquire_or_heartbeat_leadership(
  mm: &ModelManager,
  server_id: &str,
  is_leader: &AtomicBool,
  load_balancer: &LoadBalancer,
  ttl: &Duration,
  token_increment_interval: &Duration,
) {
  match DistributedLockBmc::try_acquire_or_update(
    mm,
    DistributedLockIds::SCHED_SERVER_LEADER,
    server_id,
    ttl,
    token_increment_interval,
  )
  .await
  {
    Ok(Some(_)) => {
      if !is_leader.swap(true, Ordering::SeqCst) {
        info!("Acquired leadership successfully, transitioning to leader mode");
      }
      // 执行一次平衡检查
      if let Err(e) = load_balancer.rebalance_if_needed().await {
        error!("Failed try to rebalance, error: {:?}", e);
      }
    }
    Ok(None) => {
      // 设置为 Follower 模式
      if is_leader.swap(false, Ordering::SeqCst) {
        info!("Transitioning to follower mode successfully");
      }
    }
    Err(e) => {
      error!("Leadership acquisition error: {:?}", e);
    }
  }
}
