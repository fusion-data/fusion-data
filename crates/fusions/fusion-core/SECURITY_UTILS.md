# SecurityUtils 接口说明与使用指南

SecurityUtils 提供统一的 JWT/JWE 加解密能力，配合 `fusion_web::extract_ctx` 在 Web Handler 中快速提取 `Ctx`（包装 `CtxPayload`）。

## 接口一览

```rust
pub struct SecurityUtils;

impl SecurityUtils {
  /// 使用 `CtxPayload` 构造 JWT，并进行 JWE（dir）加密，返回 token 字符串
  pub fn encrypt_jwt(key_conf: &dyn KeyConf, payload: CtxPayload) -> Result<String, Error>;

  /// 使用 `JwtPayload` 进行加密；若未包含过期时间，会依据 `KeyConf.expires_at()` 自动设置
  pub fn encrypt_jwt_payload(key_conf: &dyn KeyConf, payload: JwtPayload) -> Result<String, Error>;

  /// 解密 JWE token 为 `CtxPayload` 与 JWE Header；如过期返回 `Error::TokenExpired`
  pub fn decrypt_jwt(key_conf: &dyn KeyConf, token: &str) -> Result<(CtxPayload, JweHeader), Error>;

  /// 解密为 `JwtPayload` 与 JWE Header（底层解密），并进行过期检查
  pub fn decrypt_jwt_payload(key_conf: &dyn KeyConf, token: &str) -> Result<(JwtPayload, JweHeader), Error>;
}
```

## 典型使用

### 1) Web Handler 提取 `Ctx`

```rust
use fusion_web::extract_ctx;
use fusion_common::ctx::Ctx;
use http::request::Parts;

async fn handler(
  axum::extract::State(app): axum::extract::State<Application>,
  mut parts: Parts,
) -> WebResult<serde_json::Value> {
  let ctx: Ctx = extract_ctx(&parts, app.fusion_setting().security())?;
  let uid = ctx.user_id();
  let scopes = ctx.payload().get_strings("scopes").unwrap_or_default();
  ok_json!(serde_json::json!({"uid": uid, "scopes": scopes}))
}
```

### 2) 生成登录 token（服务端）

```rust
use fusion_core::security::SecurityUtils;
use fusion_common::ctx::CtxPayload;

fn issue_token(key_conf: &dyn KeyConf, user_id: i64, scopes: &[&str]) -> Result<String, Error> {
  let mut payload = CtxPayload::default();
  payload.set_subject(user_id.to_string());
  payload.set_strings("scopes", scopes.iter().copied());
  SecurityUtils::encrypt_jwt(key_conf, payload)
}
```

## 与 extract_ctx 的关系

`fusion_web::extract_ctx(parts, security)` 会从 `Authorization: Bearer` 或查询参数 `access_token` 获取 token，调用 `SecurityUtils::decrypt_jwt(security.pwd(), token)` 解密并返回 `Ctx`。建议在 Handler 或 Service 构造中复用该方法，以统一令牌解析逻辑。

## 备注

- 当前实现采用 JWE（dir）加解密与过期检查（基于 `KeyConf.secret_key/expires_in`）；若需要 RS256/JWKS 校验，可在后续版本扩展为签名验真与内省兜底，但请保持对上层 `extract_ctx` 的接口不变，避免应用修改。
