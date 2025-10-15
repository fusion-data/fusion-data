use axum::{
  Json,
  extract::Path,
  routing::{get, post, put},
};
use fusion_core::application::Application;
use fusion_web::{Router, WebResult, ok_json};
use fusionsql::page::PageResult;
use serde::{Deserialize, Serialize};

use crate::domain::user::{UserEntity, UserForPage, UserForUpdate, UserSvc};

// --- API Data Models ---

/// 用户密码更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPasswordUpdateRequest {
  /// 旧密码（可选，用于验证）
  pub old_password: Option<String>,
  /// 验证码（可选，用于忘记密码场景）
  pub verification_code: Option<String>,
  /// 新密码
  pub new_password: String,
}

pub fn routes() -> Router<Application> {
  Router::new()
    .route("/item/{id}", get(get_user_by_id).put(update_user_by_id))
    .route("/item/{id}/password", put(update_user_password))
    .route("/query", post(query_user))
}

async fn get_user_by_id(user_svc: UserSvc, Path(id): Path<i64>) -> WebResult<Option<UserEntity>> {
  let user = user_svc.get_by_id(id).await?;
  ok_json!(user)
}

async fn update_user_by_id(
  user_svc: UserSvc,
  Path(id): Path<i64>,
  Json(for_update): Json<UserForUpdate>,
) -> WebResult<()> {
  user_svc.update(id, for_update).await?;
  ok_json!()
}

async fn query_user(user_svc: UserSvc, Json(for_page): Json<UserForPage>) -> WebResult<PageResult<UserEntity>> {
  let paged = user_svc.find_page(for_page).await?;
  ok_json!(paged)
}

/// 用户密码更新
async fn update_user_password(
  user_svc: UserSvc,
  Path(id): Path<i64>,
  Json(password_req): Json<UserPasswordUpdateRequest>,
) -> WebResult<()> {
  // TODO: 实现密码更新逻辑
  // 1. 验证用户是否存在
  // 2. 如果提供了旧密码，验证旧密码是否正确
  // 3. 如果提供了验证码，验证验证码是否有效
  // 4. 生成新密码哈希
  // 5. 更新用户密码

  // 目前作为占位符实现
  log::info!("Password update requested for user ID: {}", id);
  ok_json!()
}
