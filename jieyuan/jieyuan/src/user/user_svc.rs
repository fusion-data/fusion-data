use axum::extract::FromRequestParts;
use fusion_common::regex;
use fusion_core::{
  DataError, Result,
  application::Application,
  security::pwd::{generate_pwd, is_strong_password, verify_pwd},
};
use fusion_web::WebError;
use fusionsql::{ModelManager, filter::OpValInt64, page::PageResult};

use jieyuan_core::model::{
  TenantUserStatus, UpdatePasswordRequest, User, UserCredential, UserCredentialForInsert, UserFilter, UserForCreate,
  UserForPage, UserForUpdate, UserRoleForCreate, UserStatus,
};

use crate::utils::model_manager_from_parts;

use super::{UserBmc, UserCredentialBmc, UserRoleBmc};
use crate::TenantUserBmc;

#[derive(Clone)]
pub struct UserSvc {
  mm: ModelManager,
}

impl UserSvc {
  /// 创建新的用户服务实例
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn create(&self, input: UserForCreate) -> Result<i64> {
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    let encrypted_pwd = if let Some(password) = input.password.as_deref() {
      if password.len() < 6 {
        return Err(DataError::bad_request("Password length cannot be less than 6 characters."));
      }
      generate_pwd(password).await?
    } else {
      let setting = Application::global().fusion_setting();
      generate_pwd(setting.security().pwd().default_pwd()).await?
    };

    // 初始状态为未关联租户
    let mut create_input = Self::validate_and_init(input)?;
    create_input.status = Some(UserStatus::Inactive);

    let id = UserBmc::create(&mm, create_input).await?;
    UserCredentialBmc::insert(&mm, UserCredentialForInsert { id, encrypted_pwd }).await?;

    mm.dbx().commit_txn().await?;
    Ok(id)
  }

  pub async fn page(&self, req: UserForPage) -> Result<PageResult<User>> {
    let page = UserBmc::page(&self.mm, req.filters, req.page).await?;
    Ok(page)
  }

  pub async fn find_option_by_id(&self, id: i64) -> Result<Option<User>> {
    let f = UserFilter { id: Some(OpValInt64::eq(id)), ..Default::default() };
    let u = UserBmc::find_unique(&self.mm, vec![f]).await?;
    Ok(u)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<User> {
    let u = UserBmc::find_by_id(&self.mm, id).await?;
    Ok(u)
  }

  pub async fn update_by_id(&self, id: i64, req: UserForUpdate) -> Result<()> {
    UserBmc::update_by_id(&self.mm, id, req).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, id: i64) -> Result<()> {
    UserBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  pub async fn get_fetch_credential(&self, req: UserFilter) -> Result<(User, UserCredential)> {
    let u = UserBmc::find_unique(&self.mm, vec![req])
      .await?
      .ok_or_else(|| DataError::not_found("User not exists."))?;
    let uc = UserCredentialBmc::find_by_id(&self.mm, u.id).await?;
    Ok((u, uc))
  }

  /// 简化的用户创建方法，用于注册
  pub async fn create_user(&self, filter: UserFilter, _encrypted_pwd: String) -> Result<i64> {
    // 从 filter 中提取用户信息
    let email = filter.email.as_ref().and_then(|op| op.eq.clone());

    let phone = filter.phone.as_ref().and_then(|op| op.eq.clone());

    let user_create = UserForCreate {
      email,
      phone,
      name: None,                         // 将在 validate_and_init 中自动生成
      status: Some(UserStatus::Inactive), // 初始状态为未关联租户
      password: None,                     // 密码已经加密，不需要再处理
    };

    self.create(user_create).await
  }

  pub async fn assign_role(&self, user_id: i64, role_ids: Vec<i64>) -> Result<()> {
    let user_roles = role_ids.into_iter().map(|role_id| UserRoleForCreate { user_id, role_id }).collect();
    UserRoleBmc::insert_many(&self.mm, user_roles).await?;
    Ok(())
  }

  /// 关联用户到租户
  pub async fn link_user_to_tenant(&self, user_id: i64, tenant_id: i64, status: TenantUserStatus) -> Result<()> {
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 创建关联
    TenantUserBmc::link_user_to_tenant(&mm, user_id, tenant_id, status).await?;

    // 更新用户状态
    self.update_user_status_based_on_tenants(&mm, user_id).await?;

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  /// 取消用户与租户的关联
  pub async fn unlink_user_from_tenant(&self, user_id: i64, tenant_id: i64) -> Result<()> {
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 删除关联
    TenantUserBmc::unlink_user_from_tenant(&mm, user_id, tenant_id).await?;

    // 更新用户状态
    self.update_user_status_based_on_tenants(&mm, user_id).await?;

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  /// 更新用户在租户中的状态
  pub async fn update_user_tenant_status(&self, user_id: i64, tenant_id: i64, status: TenantUserStatus) -> Result<()> {
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 更新关联状态
    TenantUserBmc::update_tenant_user_status(&mm, user_id, tenant_id, status).await?;

    // 更新用户状态
    self.update_user_status_based_on_tenants(&mm, user_id).await?;

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  /// 根据用户的租户关联状态更新用户状态
  async fn update_user_status_based_on_tenants(&self, mm: &ModelManager, user_id: i64) -> Result<()> {
    let active_tenant_count = TenantUserBmc::get_user_active_tenant_count(mm, user_id).await?;

    let new_status = if active_tenant_count > 0 { UserStatus::Active } else { UserStatus::Inactive };

    UserBmc::update_by_id(mm, user_id, UserForUpdate { status: Some(new_status), name: None }).await?;

    Ok(())
  }

  /// 检查用户在指定租户中是否活跃
  pub async fn is_user_active_in_tenant(&self, user_id: i64, tenant_id: i64) -> Result<bool> {
    TenantUserBmc::is_user_active_in_tenant(&self.mm, user_id, tenant_id)
      .await
      .map_err(|e| fusion_core::DataError::from(e))
  }

  /// 获取用户的所有活跃租户关联
  pub async fn get_user_active_tenants(&self, user_id: i64) -> Result<Vec<jieyuan_core::model::TenantUser>> {
    TenantUserBmc::get_user_active_tenants(&self.mm, user_id)
      .await
      .map_err(|e| fusion_core::DataError::from(e))
  }

  /// 获取包含租户信息的用户数据（用于登录验证）
  pub async fn get_user_with_tenant(
    &self,
    user_id: i64,
    tenant_id: i64,
  ) -> Result<Option<jieyuan_core::model::UserWithTenant>> {
    TenantUserBmc::get_user_with_tenant(&self.mm, user_id, tenant_id)
      .await
      .map_err(|e| fusion_core::DataError::from(e))
  }

  /// 修改密码功能
  pub async fn update_password(
    &self,
    actor_user_id: i64,
    actor_tenant_id: i64,
    target_user_id: i64,
    req: UpdatePasswordRequest,
  ) -> Result<()> {
    // 1) 校验密码复杂度
    if !is_strong_password(&req.new_password) {
      return Err(DataError::bad_request(
        "Password must be at least 8 characters long and contain uppercase, lowercase, and digits",
      ));
    }

    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    // 2) 读取并锁定目标用户（租户隔离）
    let user_credential = UserCredentialBmc::get_by_id_for_update(&mm, target_user_id, req.tenant_id)
      .await?
      .ok_or_else(|| DataError::not_found("User not found in specified tenant"))?;

    // 3) 权限判断
    let is_self = actor_user_id == target_user_id;

    // 检查操作者是否为管理员（这里简化为平台租户管理，实际应根据权限系统判断）
    let is_admin = actor_tenant_id == 1; // 平台租户管理员

    if !is_self && !is_admin {
      return Err(DataError::forbidden(
        "Only users can modify their own password or administrators can modify others' passwords",
      ));
    }

    // 4) 自助修改必须校验旧密码
    if is_self {
      let old_password = req
        .old_password
        .ok_or_else(|| DataError::bad_request("Old password is required for self-service password change"))?;

      // 校验旧密码
      verify_pwd(&old_password, &user_credential.encrypted_pwd)
        .await
        .map_err(|_| DataError::bad_request("Current password is incorrect"))?;
    }

    // 5) 生成新密码哈希（复用 generate_pwd）
    let new_hashed_pwd = generate_pwd(&req.new_password).await?;

    // 6) 事务内原子更新密码并令牌序列自增（并发下最后提交为准）
    UserCredentialBmc::update_password_and_bump_token_seq(&mm, target_user_id, &new_hashed_pwd).await?;

    mm.dbx().commit_txn().await?;
    Ok(())
  }

  /// 校验数据并进行初始化。`email` 或 `phone` 至少有一个，若两个值都设置，则只有 `email` 有效。
  ///
  /// 当 `name` 未设置时，将从 `email` 或 `phone` 中取值。
  pub fn validate_and_init(mut input: UserForCreate) -> Result<UserForCreate> {
    if let Some(email) = input.email.as_deref() {
      if !regex::is_email(email) {
        return Err(DataError::bad_request("The 'email' field is invalid"));
      }
    } else if let Some(phone) = input.phone.as_deref() {
      if !regex::is_phone(phone) {
        return Err(DataError::bad_request("The 'phone' field is invalid"));
      }
    } else {
      return Err(DataError::bad_request("At least one 'email' or 'phone' is required"));
    };

    if input.name.as_deref().is_none_or(|n| n.is_empty()) {
      input.name = match input.email.as_deref() {
        Some(email) => email.split('@').next().map(ToString::to_string),
        None => input.phone.clone(),
      };
    }

    Ok(input)
  }
}

impl FromRequestParts<Application> for UserSvc {
  type Rejection = WebError;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    let mm = model_manager_from_parts(parts, state)?;
    Ok(Self::new(mm))
  }
}
