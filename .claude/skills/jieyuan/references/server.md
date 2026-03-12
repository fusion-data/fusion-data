# jieyuan-server

服务端：OAuth、Tenant、Permission、Role、User、Access Control。

## Imports

```rust
use jieyuan_server::{
    access_control::{AuthSvc, PolicySvc},
    user::{UserSvc, UserBmc},
    tenant::{TenantSvc, TenantBmc},
    role::{RoleSvc, RoleBmc},
    permission::{PermissionSvc, PermissionBmc},
    oauth::OAuthSvc,
};
```

## AuthSvc

```rust
use hetusql::ModelManager;
use hetus::security::jwt::make_token_with_tenant;

pub struct AuthSvc {
    user_svc: UserSvc,
}

impl AuthSvc {
    pub async fn signin(&self, req: SigninRequest) -> Result<SigninResponse> {
        let (user_filter, password, tenant_id) = req.into_split();

        let (user, credential) = self.user_svc.get_fetch_credential(user_filter).await?;

        verify_pwd(&password, &credential.encrypted_pwd).await?;

        if user.status != UserStatus::Active {
            return Err(DataError::forbidden("User account is not active"));
        }

        let config = Application::global().hetu_setting();
        let token = make_token_with_tenant(config.security(), user.id, tenant_id, 0)?;

        Ok(SigninResponse {
            token,
            token_type: TokenType::Bearer,
        })
    }

    pub async fn signup(&self, req: SignupReq) -> Result<i64> {
        self.user_svc.create(req).await
    }

    pub async fn refresh_token(&self, req: RefreshTokenReq) -> Result<SigninResponse> {
        let payload = validate_token(req.token)?;

        let user = self.user_svc.get(payload.subject().parse()?).await?
            .ok_or_else(|| DataError::not_found("User not found"))?;

        let config = Application::global().hetu_setting();
        let new_token = make_token_with_tenant(
            config.security(),
            user.id,
            payload.tenant_id(),
            payload.token_seq(),
        )?;

        Ok(SigninResponse {
            token: new_token,
            token_type: TokenType::Bearer,
        })
    }

    pub async fn validate_token_with_tenant(&self, token: &str) -> Result<(i64, i64)> {
        validate_token_with_tenant(token)
    }
}
```

## UserSvc

```rust
pub struct UserSvc {
    mm: ModelManager,
}

impl UserSvc {
    pub async fn create(&self, user: UserForCreate) -> Result<i64> {
        // 检查邮箱是否已存在
        if self.find_by_email(&user.email).await?.is_some() {
            return Err(DataError::conflicted("Email already exists"));
        }

        // 哈希密码
        let encrypted_pwd = hash_password(&user.password)?;

        let entity = UserEntity {
            email: user.email,
            username: user.username,
            encrypted_pwd,
            status: UserStatus::Active,
            tenant_id: user.tenant_id,
            ..Default::default()
        };

        UserBmc::insert(&self.mm, entity).await
    }

    pub async fn get(&self, id: i64) -> Result<Option<User>> {
        UserBmc::find_by_id(&self.mm, id).await
    }

    pub async fn update(&self, id: i64, user: UserForUpdate) -> Result<()> {
        UserBmc::update_by_id(&self.mm, id, user).await
    }

    pub async fn update_password(&self, id: i64, old_pwd: &str, new_pwd: &str) -> Result<()> {
        let user = self.get_fetch_credential(UserFilter::ById(id)).await?;

        verify_pwd(old_pwd, &user.encrypted_pwd).await?;

        let new_encrypted = hash_password(new_pwd)?;
        UserBmc::update_password(&self.mm, id, new_encrypted).await
    }

    pub async fn list(&self, filter: UserFilter, page: Page) -> Result<PageResult<User>> {
        UserBmc::list(&self.mm, Some(filter), Some(page)).await
    }
}
```

## TenantSvc

```rust
pub struct TenantSvc {
    mm: ModelManager,
}

impl TenantSvc {
    pub async fn create(&self, tenant: TenantForCreate) -> Result<i64> {
        let entity = TenantEntity {
            name: tenant.name,
            code: tenant.code,
            status: TenantStatus::Active,
            ..Default::default()
        };

        TenantBmc::insert(&self.mm, entity).await
    }

    pub async fn add_user(&self, tenant_id: i64, user_id: i64, role_ids: Vec<i64>) -> Result<()> {
        self.mm.transaction(|mm| async move {
            // 添加租户用户关联
            TenantUserBmc::insert(&mm, TenantUserEntity {
                tenant_id,
                user_id,
                status: TenantUserStatus::Active,
            }).await?;

            // 分配角色
            for role_id in role_ids {
                UserRoleBmc::insert(&mm, UserRoleEntity {
                    user_id,
                    role_id,
                    tenant_id,
                }).await?;
            }

            Ok(())
        }).await
    }

    pub async fn list_users(&self, tenant_id: i64) -> Result<Vec<TenantUser>> {
        TenantUserBmc::list_by_tenant(&self.mm, tenant_id).await
    }
}
```

## RoleSvc

```rust
pub struct RoleSvc {
    mm: ModelManager,
}

impl RoleSvc {
    pub async fn create(&self, role: RoleForCreate) -> Result<i64> {
        let entity = RoleEntity {
            tenant_id: role.tenant_id,
            name: role.name,
            code: role.code,
            description: role.description,
        };

        RoleBmc::insert(&self.mm, entity).await
    }

    pub async fn assign_permissions(&self, role_id: i64, permission_ids: Vec<i64>) -> Result<()> {
        // 删除旧的关联
        RolePermissionBmc::delete_by_role(&self.mm, role_id).await?;

        // 添加新的关联
        for perm_id in permission_ids {
            RolePermissionBmc::insert(&self.mm, RolePermissionEntity {
                role_id,
                permission_id: perm_id,
            }).await?;
        }

        Ok(())
    }

    pub async fn get_permissions(&self, role_id: i64) -> Result<Vec<Permission>> {
        RolePermissionBmc::list_permissions(&self.mm, role_id).await
    }
}
```

## PermissionSvc

```rust
pub struct PermissionSvc {
    mm: ModelManager,
}

impl PermissionSvc {
    pub async fn create(&self, perm: PermissionForCreate) -> Result<i64> {
        let entity = PermissionEntity {
            tenant_id: perm.tenant_id,
            name: perm.name,
            code: perm.code,
            resource: perm.resource,
            action: perm.action,
        };

        PermissionBmc::insert(&self.mm, entity).await
    }

    pub async fn check(&self, user_id: i64, action: &str, resource: &str) -> Result<bool> {
        // 获取用户的所有角色
        let roles = UserRoleBmc::list_roles(&self.mm, user_id).await?;

        // 获取角色的所有策略
        let mut policies = Vec::new();
        for role in roles {
            let role_policies = RolePolicyBmc::list_policies(&self.mm, role.id).await?;
            policies.extend(role_policies);
        }

        // 使用策略引擎检查
        let ctx = Ctx::from_user_id(user_id);
        Ok(PolicyEngine::match_any(&policies, &ctx, action, resource, DecisionEffect::Allow))
    }
}
```

## OAuthSvc

```rust
pub struct OAuthSvc {
    http_client: reqwest::Client,
    user_svc: UserSvc,
}

impl OAuthSvc {
    pub async fn authorize_url(&self, req: OAuthAuthorizeRequest) -> Result<String> {
        let provider = self.get_provider(&req.provider)?;

        // 生成 state 和 PKCE challenge
        let state = generate_random_string(32);
        let pkce = PKCE::new();

        // 构建 URL
        let mut url = Url::parse(&provider.authorize_url)?;

        url.query_pairs_mut()
            .append_pair("client_id", &provider.client_id)
            .append_pair("redirect_uri", &req.redirect_uri.unwrap_or_default())
            .append_pair("response_type", "code")
            .append_pair("state", &state)
            .append_pair("scope", &provider.scopes.join(" "));

        if let Some(challenge) = &req.code_challenge {
            url.query_pairs_mut()
                .append_pair("code_challenge", challenge)
                .append_pair("code_challenge_method", req.code_challenge_method.as_deref().unwrap_or("S256"));
        }

        Ok(url.to_string())
    }

    pub async fn callback(&self, req: OAuthCallbackRequest) -> Result<SigninResponse> {
        // 交换 code 获取 token
        let token = self.exchange_code(&req.provider, &req.code, &req.code_verifier).await?;

        // 获取用户信息
        let user_info = self.get_user_info(&req.provider, &token.access_token).await?;

        // 查找或创建用户
        let user = self.user_svc.find_or_create_from_oauth(&req.provider, &user_info).await?;

        // 生成 JWT
        let config = Application::global().hetu_setting();
        let jwt_token = make_token_by_user_id(config.security(), user.id)?;

        Ok(SigninResponse {
            token: jwt_token,
            token_type: TokenType::Bearer,
        })
    }
}
```

## API Endpoints

```rust
use utoipa_axum::router::OpenApiRouter;

pub fn routes() -> OpenApiRouter<Application> {
    OpenApiRouter::new()
        // Auth
        .routes(utoipa_axum::routes!(signin))
        .routes(utoipa_axum::routes!(signup))
        .routes(utoipa_axum::routes!(refresh_token))
        // Users
        .routes(utoipa_axum::routes!(user_create))
        .routes(utoipa_axum::routes!(user_get))
        .routes(utoipa_axum::routes!(user_page))
        // Tenants
        .routes(utoipa_axum::routes!(tenant_create))
        .routes(utoipa_axum::routes!(tenant_list))
        // Roles
        .routes(utoipa_axum::routes!(role_create))
        .routes(utoipa_axum::routes!(role_assign_permissions))
        // Permissions
        .routes(utoipa_axum::routes!(permission_create))
        .routes(utoipa_axum::routes!(permission_check))
}

#[utoipa::path(post, path = "/signin", tag = "Auth")]
async fn signin(
    auth_svc: AuthSvc,
    Json(req): Json<SigninRequest>,
) -> WebResult<SigninResponse> {
    let response = auth_svc.signin(req).await?;
    ok_json!(response)
}
```

## Best Practices

1. **密码安全**: 使用 bcrypt 或 argon2 哈希密码
2. **PKCE**: OAuth 公共客户端必须使用 PKCE
3. **多租户**: 所有数据按 tenant_id 隔离
4. **RBAC**: 使用 Role-Permission 模型管理权限
5. **审计日志**: 记录关键操作

## Examples from Codebase

- `jieyuan/jieyuan-server/src/access_control/auth_svc.rs` - 认证服务
- `jieyuan/jieyuan-server/src/user/user_svc.rs` - 用户服务
- `jieyuan/jieyuan-server/src/tenant/tenant_svc.rs` - 租户服务
- `jieyuan/jieyuan-server/src/oauth/mod.rs` - OAuth 服务
