use std::sync::Arc;
use std::time::Duration;

use fusion_common::time::now_offset;
use fusion_core::DataError;
use fusion_core::application::Application;
use fusion_core::configuration::ConfigRegistry;
use fusion_web::config::WebConfig;
use log::{error, info};
use modelsql::ModelManager;
use tokio::sync::broadcast;
use tokio::time::interval;

use hetuflow_core::models::ServerForRegister;
use hetuflow_core::types::ServerStatus;

use crate::infra::bmc::ServerBmc;
use crate::service::TaskGenerationSvc;
use crate::setting::ServerConfig;

/// 调度器服务
pub struct SchedulerSvc {
  mm: ModelManager,
  server_config: Arc<ServerConfig>,
  task_generation_svc: Arc<TaskGenerationSvc>,
  shutdown_tx: broadcast::Sender<()>,
}

impl SchedulerSvc {
  /// 创建新的调度器服务
  pub fn new(mm: ModelManager, server_config: Arc<ServerConfig>, shutdown_tx: broadcast::Sender<()>) -> Self {
    let task_generation_svc = Arc::new(TaskGenerationSvc::new(mm.clone()));
    Self { mm, server_config, task_generation_svc, shutdown_tx }
  }

  /// 启动调度器服务
  pub async fn start(&self) -> Result<(), DataError> {
    info!("Starting Scheduler Server with server_id: {}", &self.server_config.server_id);

    // 注册服务器
    self.register_server().await?;

    // 启动生成 Task 循环
    self.start_task_generation().await?;

    // 启动心跳任务
    self.start_heartbeat_task().await;

    info!("Scheduler Service started successfully");
    Ok(())
  }

  /// 启动心跳任务
  async fn start_heartbeat_task(&self) {
    let mm = self.mm.clone();
    let server_id = self.server_config.server_id;
    let mut shutdown_rx = self.shutdown_tx.subscribe();

    tokio::spawn(async move {
      let mut interval = interval(Duration::from_secs(30));

      loop {
        tokio::select! {
          _ = interval.tick() => {
            // TODO: 实现服务器心跳更新逻辑
            if let Err(e) = Ok::<(), DataError>(()) {
              error!("Server heartbeat update failed: {}", e);
            }
          }
          _ = shutdown_rx.recv() => {
            info!("Server heartbeat shutting down");
            break;
          }
        }
      }
    });
  }

  /// 注册服务器
  async fn register_server(&self) -> Result<(), DataError> {
    info!("Registering server: {}", &self.server_config.server_id);

    let web_config: WebConfig = Application::global().get_config()?;

    let server = ServerForRegister {
      id: self.server_config.server_id,
      name: self.server_config.server_name.clone(),
      address: web_config.server_addr().to_string(),
      status: ServerStatus::Active,
    };
    ServerBmc::register(&self.mm, server).await?;
    info!("Server {} registered", &self.server_config.server_id);
    Ok(())
  }

  async fn start_task_generation(&self) -> Result<(), DataError> {
    let task_generation_svc = self.task_generation_svc.clone();
    let mut shutdown_rx = self.shutdown_tx.subscribe();
    let mut interval = interval(self.server_config.job_check_interval);
    let duration = self.server_config.job_check_duration;
    tokio::spawn(async move {
      loop {
        tokio::select! {
            _ = interval.tick() => {
                let from_time = now_offset();
                let to_time = from_time + duration;

                if let Err(e) = task_generation_svc.generate_tasks_for_schedule(from_time, to_time).await {
                    error!("Task generation failed: {}", e);
                }
            }
            _ = shutdown_rx.recv() => {
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
    // TODO: 实现释放领导权的逻辑
    info!("Releasing leadership for server: {}", self.server_config.server_id);
    Ok(())
  }
}
