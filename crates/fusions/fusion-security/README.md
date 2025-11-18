# Fusion Security

Fusion Security 是 Fusion-Data 平台的安全模块，提供认证、授权和 OAuth 集成功能。

## OAuth 集成示例

本项目包含 Gitee 和 GitHub OAuth 2.0 登录的完整示例代码。

### 功能特性

- 支持 Gitee OAuth 2.0 授权码模式
- 支持 GitHub OAuth 2.0 授权码模式
- 统一的 OAuth 客户端接口
- 可复用的数据结构和函数
- 完整的错误处理
- **自动令牌续期（内置功能）**：所有 OAuth 客户端默认支持透明的令牌续期
- **令牌存储**：内置内存存储，支持自定义存储后端
- **透明的令牌管理**：自动检测令牌过期并刷新，无需用户干预

### 运行示例

#### Gitee OAuth 示例

```bash
# 设置环境变量（可选）
export GITEE_CLIENT_ID="your_gitee_client_id"
export GITEE_CLIENT_SECRET="your_gitee_client_secret"
export GITEE_REDIRECT_URL="http://localhost:8080/callback"

# 运行示例
cargo run --example example-oauth2-gitee --features with-oauth
```

#### GitHub OAuth 示例

```bash
# 设置环境变量（可选）
export GITHUB_CLIENT_ID="your_github_client_id"
export GITHUB_CLIENT_SECRET="your_github_client_secret"
export GITHUB_REDIRECT_URL="http://localhost:8080/callback"

# 运行示例
cargo run --example example-oauth2-github --features with-oauth
```

**注意**：所有示例现在都默认支持自动续期功能，无需单独的自动续期示例。

### 使用步骤

1. **创建 OAuth 应用**
   - Gitee：访问 https://gitee.com/oauth/applications 创建应用
   - GitHub：访问 https://github.com/settings/applications/new 创建应用

2. **配置回调 URL**
   - 在 OAuth 应用设置中配置正确的回调 URL
   - 默认回调 URL: `http://localhost:8080/callback`

3. **运行示例程序**
   - 程序会生成授权 URL
   - 访问授权 URL 并完成授权
   - 输入授权码获取访问令牌和用户信息

### API 文档

#### OAuthConfig

```rust
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}
```

#### OAuthProvider

```rust
// Gitee 提供者
OAuthProvider::gitee()

// GitHub 提供者
OAuthProvider::github()
```

#### OAuthClient（简化 API）

```rust
// 创建客户端（默认启用自动续期，使用内存存储）
let client = OAuthClient::new(provider, &config)?;

// 或使用自定义令牌存储
let token_store = Arc::new(MyCustomTokenStore::new());
let client = OAuthClient::with_token_store(provider, &config, token_store)?;

// 获取授权 URL
let auth_url = client.get_authorize_url(state);

// 交换访问令牌（自动存储）
let token = client.exchange_code(code, state, user_id).await?;

// 获取用户信息（自动续期）
let user = client.get_user_info(user_id).await?;
```

**关键变化**：
- 自动续期现在是默认行为，无需特殊方法
- `exchange_code()` 方法会自动存储令牌
- `get_user_info()` 方法会自动处理令牌续期

#### UserInfo

```rust
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
}
```

#### 自动续期机制

```rust
// 自动续期是透明的，以下方法会自动处理令牌续期：
let user_info = client.get_user_info(user_id).await?;

// 令牌过期检测（提前30秒）
token.is_expired_or_expiring_soon();

// 令牌过期时间计算
token.expires_at();

// TokenStore 接口（用于自定义存储实现）
pub trait TokenStore: Send + Sync {
    fn store_token(&self, user_id: &str, token: OAuthTokenResponse) -> OAuthResult<()>;
    fn get_token(&self, user_id: &str) -> OAuthResult<Option<OAuthTokenResponse>>;
    fn remove_token(&self, user_id: &str) -> OAuthResult<()>;
    fn list_tokens(&self) -> OAuthResult<Vec<(String, OAuthTokenResponse)>>;
    fn as_any(&self) -> &dyn std::any::Any;
}

// 访问内置存储（高级用法）
let store = client.token_store();
store.list_tokens()?;
store.remove_token(user_id)?;
```

### 依赖项

- `oauth2` - OAuth 2.0 客户端实现
- `reqwest` - HTTP 客户端
- `serde` - 序列化/反序列化
- `thiserror` - 错误处理

### 安全注意事项

1. 客户端密钥应安全存储，不要硬编码在代码中
2. 生产环境中应使用 HTTPS
3. 状态参数应使用随机值防止 CSRF 攻击
4. 访问令牌应安全存储和传输
5. **自动续期机制**：刷新令牌（refresh_token）应特别小心处理，它们具有长期有效性
6. **内存存储**：默认的内存存储在应用重启后会丢失令牌，生产环境应考虑持久化存储

### 错误处理

所有 OAuth 操作都返回 `OAuthResult<T>`，可能包含以下错误类型：

- `ConfigurationError` - 配置错误
- `TokenExchangeError` - 令牌交换错误
- `UserInfoError` - 用户信息获取错误
- `RequestError` - HTTP 请求错误