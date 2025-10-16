use fusion_core::{DataError, Result, application::Application, security::pwd::verify_pwd};

use fusionsql::filter::OpValString;
use jieyuan_core::model::{
  RefreshTokenReq, SigninRequest, SigninResponse, SignupReq, TenantUserStatus, TokenType, UserStatus,
};

use crate::user::UserSvc;

use super::utils::{make_token, make_token_with_tenant, validate_token};

#[derive(Clone)]
pub struct AuthSvc {
  user_svc: UserSvc,
}

impl AuthSvc {
  /// 创建新的认证服务实例
  pub fn new(user_svc: UserSvc) -> Self {
    Self { user_svc }
  }

  pub async fn signin(&self, req: SigninRequest) -> Result<SigninResponse> {
    let (filter, password, tenant_id) = req.into_split();

    // 强制要求租户ID
    let tenant_id = tenant_id.ok_or_else(|| DataError::bad_request("tenant_id is required for login"))?;

    // 首先获取用户基本信息
    let (u, uc) = self.user_svc.get_fetch_credential(filter).await?;
    verify_pwd(&password, &uc.encrypted_pwd).await?;

    // 检查用户状态是否为活跃
    if u.status != UserStatus::Active {
      return Err(DataError::forbidden("User account is not active"));
    }

    // 检查用户在指定租户中的关联状态
    let user_with_tenant = self
      .user_svc
      .get_user_with_tenant(u.id, tenant_id)
      .await?
      .ok_or_else(|| DataError::forbidden("User is not associated with this tenant"))?;

    // 检查租户关联状态是否为活跃
    if user_with_tenant.tenant_status != TenantUserStatus::Active {
      return Err(DataError::forbidden("User access to this tenant is disabled"));
    }

    // 生成包含租户ID的令牌
    let config = Application::global().fusion_config();
    let token = make_token_with_tenant(config.security(), u.id, tenant_id)?;
    Ok(SigninResponse { token, token_type: TokenType::Bearer })
  }

  /// 用户注册
  pub async fn signup(&self, req: SignupReq) -> Result<()> {
    // 验证请求参数
    req.validate().map_err(|e| DataError::bad_request(&e))?;

    let (filter, password) = req.into_split();

    // 检查用户是否已存在
    let email = filter.email.as_ref().and_then(|op| op.eq.clone());
    let phone = filter.phone.as_ref().and_then(|op| op.eq.clone());

    // 重新构建 filter 进行用户检查
    let check_filter = jieyuan_core::model::UserFilter {
      email: email.as_ref().map(|e| OpValString::eq(e.as_str())),
      phone: phone.as_ref().map(|p| OpValString::eq(p.as_str())),
      ..Default::default()
    };

    if let Ok((_, _)) = self.user_svc.get_fetch_credential(check_filter).await {
      return Err(DataError::bad_request("User already exists"));
    }

    use jieyuan_core::model::UserForCreate;
    let user_create = UserForCreate {
      email,
      phone,
      name: None,
      status: None,             // 让 UserSvc 自动设置为 Inactive
      password: Some(password), // 让 UserSvc 自动处理密码加密
    };

    self.user_svc.create(user_create).await?;
    Ok(())
  }

  /// 用户登出 - 将 token 加入黑名单
  pub async fn signout(&self, _token: &str) -> Result<()> {
    // TODO: 实现将 token 加入黑名单的逻辑
    // 这里可以调用 utils 中的黑名单相关函数
    // 目前简单返回成功
    Ok(())
  }

  /// 刷新令牌
  pub async fn refresh_token(&self, req: RefreshTokenReq) -> Result<SigninResponse> {
    // 验证 refresh token 并提取用户信息
    let user_id = validate_token(&req.refresh_token)?;

    // 生成新的 access token
    let config = Application::global().fusion_config();
    let token = make_token(config.security(), user_id)?;
    Ok(SigninResponse { token, token_type: TokenType::Bearer })
  }
}
