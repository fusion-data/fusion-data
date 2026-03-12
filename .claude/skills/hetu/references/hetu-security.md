# hetu-security

安全模块：JWT 认证、OAuth2 支持。

## Imports

```rust
use hetus::security::jwt::{make_token, make_token_by_user_id, validate_token};
use hetus::security::oauth::{
    OAuthClient, OAuthConfig, OAuthProvider, OAuthTokenResponse,
    UserInfo, TokenStore, MemoryTokenStore,
};
use hetus::core::configuration::SecuritySetting;
```

## JWT

### 创建 Token

```rust
use hetus::security::jwt::{make_token, make_token_by_user_id};
use hetus::common::ctx::CtxPayload;
use hetus::core::configuration::SecuritySetting;

// 使用 CtxPayload
let mut payload = CtxPayload::default();
payload.set_subject("user_123");
payload.set_tenant_id(456);
payload.set_expires_at(expire_time);

let token = make_token(&security_setting, payload)?;

// 简化：通过 user_id 创建
let token = make_token_by_user_id(&security_setting, "user_123")?;
```

### 验证 Token

```rust
use hetus::security::jwt::validate_token;

let payload = validate_token(&security_setting, token)?;
let user_id: i64 = payload.subject().parse()?;
let tenant_id: Option<i64> = payload.tenant_id();
```

### SecuritySetting 配置

```toml
[hetu.security]
jwt_secret = "your-secret-key"
jwt_issuer = "myapp"
jwt_audience = "myapp-users"
token_expire_seconds = 3600
```

## OAuth2

### 配置

```rust
use hetus::security::oauth::{OAuthConfig, OAuthProvider};

let config = OAuthConfig {
    client_id: "your_client_id".to_string(),
    client_secret: "your_client_secret".to_string(),
    redirect_url: "http://localhost:8080/callback".to_string(),
    scopes: vec!["user:email".to_string()],
};
```

### OAuthClient

```rust
use hetus::security::oauth::{OAuthClient, OAuthProvider};

// GitHub OAuth
let client = OAuthClient::new(OAuthProvider::github(), &config)?;

// 获取授权 URL
let auth_url = client.get_authorize_url("random_state", None, None);
// 重定向用户到 auth_url

// 交换 code 获取 token
let token = client.exchange_code("auth_code", "state").await?;

// 刷新 token
let new_token = client.refresh_token(&token.refresh_token).await?;
```

### OAuthProvider

```rust
pub enum OAuthProvider {
    Wechat,    // 微信
    Alipay,    // 支付宝
    Weibo,     // 微博
    Gitee,     // Gitee
    Github,    // GitHub
    Google,    // Google
}

impl OAuthProvider {
    pub fn github() -> Self { Self::Github }
    pub fn gitee() -> Self { Self::Gitee }
}
```

### PKCE 支持

```rust
use hetus::security::oauth::PKCE;

// 生成 PKCE challenge
let pkce = PKCE::new();
let code_challenge = pkce.challenge();
let code_verifier = pkce.verifier();

// 授权请求
let auth_url = client.get_authorize_url(
    "state",
    Some(&code_challenge),
    Some("S256"),
);

// 交换 token 时提供 verifier
let token = client.exchange_code_with_pkce(
    "auth_code",
    "state",
    &code_verifier,
).await?;
```

### TokenStore

```rust
use hetus::security::oauth::{TokenStore, MemoryTokenStore};

// 内存存储（开发/测试）
let store = MemoryTokenStore::new();

// 存储 token
store.save("user_id", token).await?;

// 获取 token
let token = store.get("user_id").await?;

// 删除 token
store.delete("user_id").await?;

// 自定义存储（生产）
impl TokenStore for PostgresTokenStore {
    async fn save(&self, user_id: &str, token: &OAuthTokenResponse) -> Result<()>;
    async fn get(&self, user_id: &str) -> Result<Option<OAuthTokenResponse>>;
    async fn delete(&self, user_id: &str) -> Result<()>;
}
```

## 认证中间件集成

```rust
use axum::middleware::{from_fn, from_fn_with_state};
use axum::{Request, Next, Response};
use http::header::AUTHORIZATION;

async fn auth_middleware(
    State(settings): State<SecuritySetting>,
    mut req: Request,
    next: Next,
) -> Result<Response, WebError> {
    let auth_header = req.headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| WebError::unauthorized("Missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| WebError::unauthorized("Invalid token format"))?;

    let payload = validate_token(&settings, token)?;

    // 将用户信息注入请求扩展
    let ctx = Ctx::from_payload(payload)?;
    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}
```

## 密码处理

```rust
use hetus::security::password::{hash_password, verify_password};

// 哈希密码
let hashed = hash_password("plain_password")?;

// 验证密码
let valid = verify_password("plain_password", &hashed)?;
```

## Best Practices

1. **Token 存储**: 生产环境使用持久化 `TokenStore`（如 Redis/PostgreSQL）
2. **PKCE**: 公共客户端（SPA、移动端）必须使用 PKCE
3. **Token 刷新**: OAuthClient 自动处理 token 刷新
4. **密钥管理**: 使用环境变量或密钥管理服务存储 `jwt_secret`
5. **HTTPS**: 生产环境必须使用 HTTPS

## Examples from Codebase

- `crates/hetu-security/src/jwt.rs` - JWT 实现
- `crates/hetu-security/src/oauth/mod.rs` - OAuth 实现
- `jieyuan/jieyuan-server/src/access_control/auth_svc.rs` - 认证服务
