use chrono::{DateTime, FixedOffset, Utc};
use fusion_common::env::get_env;
use fusion_core::DataError;
use fusionsql::ModelManager;
use log::{debug, error, info, warn};
use mea::shutdown::ShutdownRecv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::time::Duration;

use super::{UserBmc, UserForUpdate, UserStatus};

/// 用户变更查询请求
#[derive(Debug, Clone, Serialize)]
pub struct UserChangeQueryReq {
  /// 上次查询时间
  pub since: DateTime<FixedOffset>,
  /// 查询的最大数量
  pub limit: Option<i64>,
}

/// 用户变更信息
#[derive(Debug, Clone, Deserialize)]
pub struct UserChangeInfo {
  /// 用户ID
  pub user_id: i64,
  /// 变更时间
  pub changed_at: DateTime<FixedOffset>,
  /// 变更类型
  pub change_type: UserChangeType,
  /// 用户数据（完整信息）
  pub user_data: UserChangeData,
}

/// 用户变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum UserChangeType {
  /// 创建用户
  Created = 1,
  /// 更新用户
  Updated = 2,
  /// 删除用户
  Deleted = 3,
  /// 状态变更
  StatusChanged = 4,
}

/// 用户变更数据
#[derive(Debug, Clone, Deserialize)]
pub struct UserChangeData {
  pub id: i64,
  pub tenant_id: i64,
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub status: UserStatus,
  pub updated_at: Option<DateTime<FixedOffset>>,
  pub updated_by: Option<i64>,
}

/// 用户同步结果
#[derive(Debug, Clone)]
pub struct UserSyncResult {
  /// 查询时间范围
  pub query_since: DateTime<FixedOffset>,
  /// 查询时间范围
  pub query_until: DateTime<FixedOffset>,
  /// 处理的变更数量
  pub processed_changes: i64,
  /// 成功同步数量
  pub successful_syncs: i64,
  /// 失败同步数量
  pub failed_syncs: i64,
  /// 下次查询时间
  pub next_since: DateTime<FixedOffset>,
}

/// 用户同步服务
#[derive(Clone)]
pub struct UserSyncSvc {
  mm: ModelManager,
  http: Client,
  jieyuan_base_url: String,
}

impl UserSyncSvc {
  /// 创建新的用户同步服务实例
  pub fn new(mm: ModelManager) -> Result<Self, DataError> {
    let jieyuan_base_url =
      get_env("JIEYUAN_BASE_URL").map_err(|_| DataError::server_error("JIEYUAN_BASE_URL not found"))?;
    Ok(Self {
      mm,
      http: Client::builder().timeout(Duration::from_secs(30)).build().unwrap_or_default(),
      jieyuan_base_url,
    })
  }

  /// 执行用户同步
  pub async fn sync_users(&self, since: DateTime<FixedOffset>) -> Result<UserSyncResult, DataError> {
    info!("Starting user sync since: {}", since);

    let query_until = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());
    let mut processed_changes = 0i64;
    let mut successful_syncs = 0i64;
    let mut failed_syncs = 0i64;

    // 构建查询请求
    let query_req = UserChangeQueryReq {
      since,
      limit: Some(100), // 每次最多处理100个变更
    };

    // 查询用户变更
    let changes = self.query_user_changes(query_req).await?;
    processed_changes = changes.len() as i64;

    debug!("Found {} user changes to process", changes.len());

    for change in changes {
      match self.process_user_change(&change).await {
        Ok(_) => {
          successful_syncs += 1;
          debug!("Successfully processed user change for user_id: {}", change.user_id);
        }
        Err(e) => {
          failed_syncs += 1;
          error!("Failed to process user change for user_id: {}: {:?}", change.user_id, e);
        }
      }
    }

    // 计算下次查询时间（当前时间+5分钟缓冲）
    let next_since = query_until + chrono::Duration::minutes(5);

    let result =
      UserSyncResult { query_since: since, query_until, processed_changes, successful_syncs, failed_syncs, next_since };

    info!(
      "User sync completed: processed={}, successful={}, failed={}, next_since={}",
      result.processed_changes, result.successful_syncs, result.failed_syncs, result.next_since
    );

    Ok(result)
  }

  /// 查询用户变更
  async fn query_user_changes(&self, req: UserChangeQueryReq) -> Result<Vec<UserChangeInfo>, DataError> {
    let url = format!("{}/api/v1/auth/user-changes", self.jieyuan_base_url);

    let response = self
      .http
      .get(&url)
      .query(&req)
      .send()
      .await
      .map_err(|e| DataError::server_error(format!("Failed to query user changes: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(DataError::server_error(format!("User change query failed: {}", error_text)));
    }

    let changes: Vec<UserChangeInfo> = response
      .json()
      .await
      .map_err(|e| DataError::server_error(format!("Failed to parse user changes: {}", e)))?;

    Ok(changes)
  }

  /// 处理单个用户变更
  async fn process_user_change(&self, change: &UserChangeInfo) -> Result<(), DataError> {
    match change.change_type {
      UserChangeType::Created => self.handle_user_created(&change.user_data).await,
      UserChangeType::Updated => self.handle_user_updated(&change.user_data).await,
      UserChangeType::Deleted => self.handle_user_deleted(change.user_id).await,
      UserChangeType::StatusChanged => self.handle_user_status_changed(&change.user_data).await,
    }
  }

  /// 处理用户创建
  async fn handle_user_created(&self, user_data: &UserChangeData) -> Result<(), DataError> {
    info!("Handling user created for user_id: {}", user_data.id);

    // 检查用户是否已存在
    if let Some(existing_user) = UserBmc::get_by_id(&self.mm, user_data.id).await? {
      warn!("User {} already exists, skipping creation", user_data.id);
      return Ok(());
    }

    // 创建用户实体（不包含密码，因为密码由jieyuan管理）
    let user_create = crate::domain::user::UserForCreate {
      tenant_id: user_data.tenant_id,
      email: user_data.email.clone(),
      phone: user_data.phone.clone(),
      name: user_data.name.clone(),
      status: user_data.status,
      password: "".to_string(), // 密码由jieyuan管理，这里使用空字符串
    };

    UserBmc::create(&self.mm, user_create).await?;
    info!("Successfully created user {}", user_data.id);

    Ok(())
  }

  /// 处理用户更新
  async fn handle_user_updated(&self, user_data: &UserChangeData) -> Result<(), DataError> {
    info!("Handling user updated for user_id: {}", user_data.id);

    // 检查用户是否存在
    let existing_user = UserBmc::get_by_id(&self.mm, user_data.id)
      .await?
      .ok_or_else(|| DataError::not_found(format!("User {} not found", user_data.id)))?;

    // 构建更新数据
    let user_update = UserForUpdate {
      email: if existing_user.email != user_data.email { Some(user_data.email.clone()) } else { None },
      phone: if existing_user.phone != user_data.phone { user_data.phone.clone() } else { None },
      name: if existing_user.name != user_data.name { user_data.name.clone() } else { None },
      status: if existing_user.status != user_data.status { Some(user_data.status) } else { None },
      update_mask: None, // 自动检测变更字段
    };

    UserBmc::update_by_id(&self.mm, user_data.id, user_update).await?;
    info!("Successfully updated user {}", user_data.id);

    Ok(())
  }

  /// 处理用户删除
  async fn handle_user_deleted(&self, user_id: i64) -> Result<(), DataError> {
    info!("Handling user deleted for user_id: {}", user_id);

    // 检查用户是否存在
    let _existing_user = UserBmc::get_by_id(&self.mm, user_id)
      .await?
      .ok_or_else(|| DataError::not_found(format!("User {} not found", user_id)))?;

    // 软删除用户（将状态设置为禁用）
    let user_update = UserForUpdate { status: Some(UserStatus::Disabled), ..Default::default() };

    UserBmc::update_by_id(&self.mm, user_id, user_update).await?;
    info!("Successfully soft-deleted user {}", user_id);

    Ok(())
  }

  /// 处理用户状态变更
  async fn handle_user_status_changed(&self, user_data: &UserChangeData) -> Result<(), DataError> {
    info!("Handling user status changed for user_id: {}, status: {:?}", user_data.id, user_data.status);

    // 检查用户是否存在
    let existing_user = UserBmc::get_by_id(&self.mm, user_data.id)
      .await?
      .ok_or_else(|| DataError::not_found(format!("User {} not found", user_data.id)))?;

    // 如果状态相同，跳过更新
    if existing_user.status == user_data.status {
      debug!("User {} status unchanged, skipping update", user_data.id);
      return Ok(());
    }

    // 更新用户状态
    let user_update = UserForUpdate { status: Some(user_data.status), ..Default::default() };

    UserBmc::update_by_id(&self.mm, user_data.id, user_update).await?;
    info!("Successfully updated user {} status to {:?}", user_data.id, user_data.status);

    Ok(())
  }

  /// 启动定期同步任务
  pub async fn start_periodic_sync(&self, shutdown_rx: ShutdownRecv) -> Result<(), DataError> {
    info!("Starting periodic user sync service");

    // 获取上次同步时间
    let last_sync_time = self.get_last_sync_time().await?;
    let mut since = last_sync_time;

    loop {
      tokio::select! {
        _ = shutdown_rx.is_shutdown() => {
          info!("User sync service received shutdown signal");
          return Ok(());
        }
        result = self.sync_users(since) => {
          match result {
            Ok(result) => {
              // 更新下次同步时间
              since = result.next_since;

              // 保存同步时间戳
              if let Err(e) = self.save_last_sync_time(result.query_until).await {
                error!("Failed to save last sync time: {:?}", e);
              }
            }
            Err(e) => {
              error!("User sync failed: {:?}", e);
              // 如果同步失败，使用当前时间继续下次同步
              since = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());
            }
          }
        }
      }
      // 等待5分钟后进行下次同步
      tokio::time::sleep(Duration::from_secs(300)).await;
    }
  }

  /// 获取上次同步时间
  async fn get_last_sync_time(&self) -> Result<DateTime<FixedOffset>, DataError> {
    // TODO: 从配置或数据库中获取上次同步时间
    // 这里暂时使用24小时前作为默认值
    let default_time = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap()) - chrono::Duration::hours(24);

    Ok(default_time)
  }

  /// 保存最后同步时间
  async fn save_last_sync_time(&self, sync_time: DateTime<FixedOffset>) -> Result<(), DataError> {
    // TODO: 将同步时间保存到配置或数据库中
    debug!("Saving last sync time: {}", sync_time);
    Ok(())
  }
}
