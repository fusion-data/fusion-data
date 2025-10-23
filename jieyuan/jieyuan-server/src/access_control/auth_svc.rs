use axum::extract::FromRequestParts;
use fusions::core::{DataError, Result, application::Application, security::pwd::verify_pwd};
use fusions::web::WebError;

use jieyuan_core::model::{
  RefreshTokenReq, SigninRequest, SigninResponse, SignupReq, TenantUserStatus, TokenType, UserFilter, UserStatus,
};

use crate::user::UserSvc;

use super::auth_utils::*;

/// 认证服务
///
/// 提供用户认证相关功能，包括登录、注册、令牌验证等。
/// 这是 IAM 系统的核心组件之一，与策略和资源映射紧密集成。
#[derive(Clone)]
pub struct AuthSvc {
  user_svc: UserSvc,
}

impl AuthSvc {
  /// 创建新的认证服务实例
  ///
  /// # Arguments
  /// * `user_svc` - 用户服务实例
  ///
  /// # Returns
  /// 认证服务实例
  pub fn new(user_svc: UserSvc) -> Self {
    Self { user_svc }
  }

  /// 用户登录
  ///
  /// 验证用户凭据并生成访问令牌。
  ///
  /// # Arguments
  /// * `req` - 登录请求，包含用户标识和密码
  ///
  /// # Returns
  /// 包含访问令牌的登录响应
  ///
  /// # Errors
  /// 如果认证失败或令牌生成失败
  pub async fn signin(&self, req: SigninRequest) -> Result<SigninResponse> {
    let (user_filter, password, tenant_id) = req.into_split();

    // 获取用户信息
    let (user, credential) = self.user_svc.get_fetch_credential(user_filter).await?;

    // 验证密码
    verify_pwd(&password, &credential.encrypted_pwd)
      .await
      .map_err(|_| DataError::unauthorized("Invalid credentials"))?;

    // 检查用户状态
    if user.status != UserStatus::Active {
      return Err(DataError::forbidden("User account is not active"));
    }

    // 如果指定了租户ID，验证租户关联
    let final_tenant_id = if let Some(tenant_id) = tenant_id {
      // 检查用户是否属于该租户
      let user_with_tenant = self
        .user_svc
        .get_user_with_tenant(user.id, tenant_id)
        .await?
        .ok_or_else(|| DataError::forbidden("User is not associated with this tenant"))?;

      // 检查租户关联状态是否为活跃
      if user_with_tenant.tenant_status != TenantUserStatus::Active {
        return Err(DataError::forbidden("User access to this tenant is disabled"));
      }

      Some(tenant_id)
    } else {
      // 如果没有指定租户ID，使用用户的默认租户
      None
    };

    // 生成访问令牌
    let token = if let Some(tenant_id) = final_tenant_id {
      // 如果有租户ID，生成包含租户信息的令牌
      let config = Application::global().fusion_setting();
      make_token_with_tenant(config.security(), user.id, tenant_id, 0)?
    } else {
      // 生成基本令牌
      let config = Application::global().fusion_setting();
      make_token(config.security(), user.id)?
    };

    Ok(SigninResponse { token, token_type: TokenType::Bearer })
  }

  /// 刷新访问令牌
  ///
  /// 使用刷新令牌获取新的访问令牌。
  /// 同时会验证用户状态和租户关联状态。
  pub async fn refresh_token(&self, req: RefreshTokenReq) -> Result<SigninResponse> {
    // 验证刷新令牌并获取用户信息
    let (user_id, tenant_id) = validate_token_with_tenant(&req.refresh_token)?;

    // 获取用户信息
    let user_filter = UserFilter { id: Some(fusionsql::filter::OpValInt64::eq(user_id)), ..Default::default() };

    let (user, _credential) = self.user_svc.get_fetch_credential(user_filter).await?;

    // 检查用户状态
    if user.status != UserStatus::Active {
      return Err(DataError::forbidden("User account is not active"));
    }

    // 验证租户关联（因为从token中提取的tenant_id总是有效的）
    let user_with_tenant = self
      .user_svc
      .get_user_with_tenant(user.id, tenant_id)
      .await?
      .ok_or_else(|| DataError::forbidden("User is not associated with this tenant"))?;

    if user_with_tenant.tenant_status != TenantUserStatus::Active {
      return Err(DataError::forbidden("User access to this tenant is disabled"));
    }

    // 生成新的访问令牌（包含租户信息）
    let config = Application::global().fusion_setting();
    let token = make_token_with_tenant(config.security(), user.id, tenant_id, 0)?;

    Ok(SigninResponse { token, token_type: TokenType::Bearer })
  }

  /// 用户注册
  ///
  /// 创建新用户账户。根据系统配置可能需要管理员审批。
  ///
  /// # Arguments
  /// * `_req` - 注册请求
  ///
  /// # Returns
  /// 如果注册成功返回空值
  ///
  /// # Errors
  /// 如果注册失败
  pub async fn signup(&self, _req: SignupReq) -> Result<()> {
    // TODO: 实现用户注册逻辑，可能需要管理员审批
    // 1. 验证请求数据
    // 2. 检查邮箱/手机号是否已存在
    // 3. 加密密码
    // 4. 创建用户记录
    // 5. 可能需要发送验证邮件或短信
    todo!("实现用户注册逻辑，可能需要管理员审批");
  }

  /// 用户登出
  ///
  /// 注销用户令牌。在实际实现中，可能需要将令牌加入黑名单。
  ///
  /// # Arguments
  /// * `_token` - 要注销的令牌
  ///
  /// # Returns
  /// 如果登出成功返回空值
  ///
  /// # Errors
  /// 如果登出失败
  pub async fn signout(&self, _token: &str) -> Result<()> {
    // TODO: 实现令牌注销逻辑
    // 1. 将令牌加入黑名单
    // 2. 或者更新用户的令牌序列使旧令牌失效
    // 3. 记录登出日志
    Ok(())
  }

  /// 验证令牌并提取用户ID
  ///
  /// 解析JWT令牌并验证其有效性。
  ///
  /// # Arguments
  /// * `token` - JWT令牌字符串
  ///
  /// # Returns
  /// 用户ID
  ///
  /// # Errors
  /// 如果令牌无效或解析失败
  pub async fn validate_token(&self, token: &str) -> Result<i64> {
    validate_token(token)
  }

  /// 验证令牌并提取用户ID和租户ID
  ///
  /// 解析JWT令牌并验证其有效性，同时提取租户信息。
  ///
  /// # Arguments
  /// * `token` - JWT令牌字符串
  ///
  /// # Returns
  /// 元组 (用户ID, 租户ID)
  ///
  /// # Errors
  /// 如果令牌无效或解析失败
  pub async fn validate_token_with_tenant(&self, token: &str) -> Result<(i64, i64)> {
    validate_token_with_tenant(token)
  }
}

/// 从请求中提取认证上下文的服务实现
impl FromRequestParts<fusions::core::application::Application> for AuthSvc {
  type Rejection = WebError;

  async fn from_request_parts(
    _parts: &mut axum::http::request::Parts,
    state: &fusions::core::application::Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    let mm = state.get_component::<fusionsql::ModelManager>().unwrap();
    let user_svc = UserSvc::new(mm);
    Ok(AuthSvc::new(user_svc))
  }
}
