# jieyuan-core

核心库：认证模型、策略引擎、上下文扩展、Web 中间件。

## Imports

```rust
use jieyuan_core::{
    model::{
        // 认证
        SigninRequest, SigninResponse, SignupReq, RefreshTokenReq, TokenType,
        OAuthProvider, OAuthAuthorizeRequest, OAuthTokenRequest, OAuthTokenResponse,
        // 用户
        User, UserFilter, UserStatus, UserForCreate, UserForUpdate,
        // 租户
        Tenant, TenantFilter, TenantUserStatus,
        // 权限
        Permission, Role, PolicyEntity, PolicyDocument, DecisionEffect,
        // 上下文扩展
        CtxExt, TenantAccessMode,
    },
    web::{
        path_authz::path_authz_middleware,
        client::JieyuanClient,
    },
};
```

## Auth Models

### SigninRequest / SigninResponse

```rust
pub struct SigninRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
    pub tenant_id: Option<i64>,
}

pub struct SigninResponse {
    pub token: String,
    pub token_type: TokenType,
}

pub enum TokenType {
    Unspecified = 0,
    Bearer = 1,
}
```

### OAuth Models

```rust
pub enum OAuthProvider {
    Wechat = 1,
    Alipay = 2,
    Weibo = 3,
    Gitee = 4,
    Github = 5,
}

pub struct OAuthAuthorizeRequest {
    pub provider: OAuthProvider,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,      // PKCE
    pub code_challenge_method: Option<String>, // S256
}

pub struct OAuthTokenRequest {
    pub provider: OAuthProvider,
    pub code: String,
    pub code_verifier: Option<String>,       // PKCE
    pub redirect_uri: Option<String>,
}

pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i32>,
    pub refresh_token: Option<String>,
}
```

## Policy Engine

### PolicyEntity

```rust
pub struct PolicyEntity {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,
    pub effect: DecisionEffect,  // Allow / Deny
    pub actions: Vec<String>,     // ["read", "write", "*"]
    pub resources: Vec<String>,   // ["users/*", "posts/{id}"]
    pub conditions: Option<serde_json::Value>,
}
```

### PolicyEngine

```rust
pub struct PolicyEngine;

impl PolicyEngine {
    /// 检查是否有任何策略允许
    pub fn match_any(
        policies: &[PolicyEntity],
        ctx: &Ctx,
        action: &str,
        resource: &str,
        target_effect: DecisionEffect,
    ) -> bool {
        policies.iter().any(|p| {
            Self::match_policy(p, ctx, action, resource, target_effect)
        })
    }

    /// 检查单个策略是否匹配
    pub fn match_policy(
        policy: &PolicyEntity,
        ctx: &Ctx,
        action: &str,
        resource: &str,
        target_effect: DecisionEffect,
    ) -> bool {
        policy.effect == target_effect
            && Self::match_patterns(&policy.actions, action)
            && Self::match_patterns(&policy.resources, resource)
            && Self::evaluate_condition(&policy.conditions, ctx)
    }

    /// 通配符匹配
    pub fn match_patterns(patterns: &[String], target: &str) -> bool {
        patterns.iter().any(|p| {
            if p == "*" {
                return true;
            }
            // 支持通配符匹配
            glob_match(p, target)
        })
    }

    /// 条件求值
    fn evaluate_condition(condition: &Option<Value>, ctx: &Ctx) -> bool {
        match condition {
            None => true,
            Some(cond) => {
                // 求值条件表达式
                // 例如: {"tenant_id": "${ctx.tenant_id}"}
                evaluate_conditional_expression(cond, ctx)
            }
        }
    }
}
```

## CtxExt Trait

```rust
pub trait CtxExt {
    fn has_role(&self, role: &str) -> bool;
    fn is_platform_admin(&self) -> bool;
    fn tenant_access_mode(&self) -> TenantAccessMode;
    fn managed_tenant_ids(&self) -> Vec<String>;
    fn can_access_tenant(&self, tenant_id: i64) -> bool;
    fn token_seq(&self) -> i32;
    fn request_method(&self) -> &str;
    fn request_path(&self) -> &str;
    fn client_ip(&self) -> &str;
    fn roles(&self) -> Vec<&str>;
}

impl CtxExt for Ctx {
    fn has_role(&self, role: &str) -> bool {
        self.roles().contains(&role)
    }

    fn is_platform_admin(&self) -> bool {
        self.has_role("platform_admin")
    }

    fn tenant_access_mode(&self) -> TenantAccessMode {
        // 从上下文获取租户访问模式
        self.get("tenant_access_mode")
            .unwrap_or(TenantAccessMode::Single)
    }

    fn can_access_tenant(&self, tenant_id: i64) -> bool {
        match self.tenant_access_mode() {
            TenantAccessMode::All => true,
            TenantAccessMode::Managed => {
                self.managed_tenant_ids().contains(&tenant_id.to_string())
            }
            TenantAccessMode::Single => {
                self.tenant_id() == tenant_id
            }
        }
    }
}

pub enum TenantAccessMode {
    All,      // 平台管理员：所有租户
    Managed,  // 管理员：管理的租户
    Single,   // 普通用户：单个租户
}
```

## Web Middleware

### path_authz_middleware

```rust
use jieyuan_core::web::path_authz_middleware;
use axum::middleware::from_fn_with_state;

pub async fn path_authz_middleware(
    State(app): State<Application>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, WebError> {
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| WebError::unauthorized("missing Authorization header"))?;

    let path_code = extract_path_code(&req);
    let request_ip = extract_client_ip(&req);

    let jieyuan_client = app.component::<JieyuanClient>();
    let authz_response = jieyuan_client
        .authorize(auth_header, AuthorizeRequest::new(path_code).with_request_ip(request_ip))
        .await?;

    if let Some(ctx) = authz_response.ctx {
        req.extensions_mut().insert(ctx);
    }

    Ok(next.run(req).await)
}

// 使用
let router = Router::new()
    .route("/api/users", get(list_users))
    .layer(from_fn_with_state(app, path_authz_middleware));
```

### JieyuanClient

```rust
pub struct JieyuanClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl JieyuanClient {
    pub async fn authorize(
        &self,
        auth_header: &str,
        request: AuthorizeRequest,
    ) -> Result<AuthorizeResponse> {
        let response = self.http_client
            .post(&format!("{}/api/v1/authorize", self.base_url))
            .header(header::AUTHORIZATION, auth_header)
            .json(&request)
            .send()
            .await?;

        let authz: AuthorizeResponse = response.json().await?;
        Ok(authz)
    }
}
```

## Best Practices

1. **PKCE**: 公共客户端（SPA、移动端）必须使用 PKCE
2. **策略最小权限**: 只授予必要的权限
3. **上下文传递**: 始终通过 Ctx 传递用户信息
4. **远程验证**: 使用 JieyuanClient 进行集中授权验证

## Examples from Codebase

- `jieyuan/jieyuan-core/src/model/auth.rs` - 认证模型
- `jieyuan/jieyuan-core/src/model/policy_engine.rs` - 策略引擎
- `jieyuan/jieyuan-core/src/model/ctx_ext.rs` - Ctx 扩展
- `jieyuan/jieyuan-core/src/web/middleware/path_authz.rs` - 授权中间件
