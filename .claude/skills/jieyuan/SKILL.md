---
name: jieyuan
description: Jieyuan IAM access control, OAuth2, PKCE, JWT, tenant, permission, role, policy engine, authentication, authorization, 访问控制
globs:
  - "jieyuan/**/*.rs"
---

# Jieyuan - IAM 访问控制

## Quick Reference

| Module | Import | Key Types |
|--------|--------|-----------|
| Core | `use jieyuan_core::*;` | `SigninRequest`, `OAuthProvider`, `PolicyEngine`, `CtxExt` |
| Server | `use jieyuan_server::*;` | `AuthSvc`, `UserSvc`, `TenantSvc`, `RoleSvc` |

## Core Types

### Auth Models
```rust
pub enum TokenType { Unspecified = 0, Bearer = 1 }

pub struct SigninRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
    pub tenant_id: Option<i64>,
}

pub enum OAuthProvider {
    Wechat = 1, Alipay = 2, Weibo = 3,
    Gitee = 4, Github = 5,
}
```

### Policy Engine
```rust
pub enum Decision { Allow, Deny }

impl PolicyEngine {
    pub fn match_any(policies: &[PolicyEntity], ctx: &Ctx, action: &str, resource: &str) -> bool;
    pub fn match_policy(policy: &PolicyEntity, ctx: &Ctx, action: &str, resource: &str) -> bool;
}
```

## Core Patterns

### Authentication
```rust
use jieyuan_server::access_control::AuthSvc;

pub struct AuthSvc {
    user_svc: UserSvc,
}

impl AuthSvc {
    pub async fn signin(&self, req: SigninRequest) -> Result<SigninResponse> {
        let (user, credential) = self.user_svc.get_fetch_credential(user_filter).await?;

        verify_pwd(&password, &credential.encrypted_pwd).await?;

        let token = make_token_with_tenant(config.security(), user.id, tenant_id, 0)?;

        Ok(SigninResponse { token, token_type: TokenType::Bearer })
    }
}
```

### CtxExt Trait
```rust
use jieyuan_core::model::CtxExt;

// 扩展 Ctx 的方法
let ctx: Ctx = ...;

if ctx.is_platform_admin() {
    // 平台管理员
}

if ctx.has_role("admin") {
    // 有 admin 角色
}

if ctx.can_access_tenant(tenant_id) {
    // 可以访问租户
}

let mode = ctx.tenant_access_mode();
// TenantAccessMode::All / ::Managed / ::Single
```

### Authorization Middleware
```rust
use jieyuan_core::web::path_authz_middleware;

let router = Router::new()
    .route("/api/users", get(list_users))
    .layer(from_fn_with_state(app, path_authz_middleware));
```

## References (按需加载)

- [core](references/core.md) - model, utils, web/middleware
- [server](references/server.md) - oauth, tenant, permission, role, user

## Related Skills
- `hetu`: 核心库模式，特别是 hetu-security
- `sql-database`: BMC/Service 层
