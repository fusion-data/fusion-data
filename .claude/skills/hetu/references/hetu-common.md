# hetu-common

通用工具库：上下文、时间、UUID、错误处理、序列化。

## Imports

```rust
use hetus::common::{
    ctx::{Ctx, CtxPayload, CtxError},
    time::{now_utc, now_offset, now, OffsetDateTime, UtcDateTime, LocalDateTime},
    uuid::Uuid,  // feature: with-uuid
    error::{Error, DataError, DataResult},
    model::{WrapperResult, IdResult, IdI64Result, IdUuidResult},
    serde::{deser_default_true, deser_default_false},
};
use hetus::common::ahash::{HashMap, HashSet};  // 高性能 HashMap
```

## Core Types

### DataError - 业务错误

> **重要**: `DataError` 是核心业务错误类型，定义在 `hetu-common` 中。

```rust
/// 数据（业务）错误，兼容 jsonrpc error
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataError {
  pub code: i32,                              // HTTP 状态码或业务错误码
  pub message: String,                        // 错误消息
  #[serde(skip_serializing_if = "Option::is_none")]
  pub detail: Option<serde_json::Value>,      // 详细信息（可选）
  #[serde(skip)]
  pub source: Option<Box<dyn Error + Send + Sync>>, // 源错误（不序列化）
}

// 构造方法
impl DataError {
  pub fn bad_request(msg: impl Into<String>) -> Self;    // 400
  pub fn not_found(msg: impl Into<String>) -> Self;      // 404
  pub fn conflicted(msg: impl Into<String>) -> Self;     // 409
  pub fn unauthorized(msg: impl Into<String>) -> Self;   // 401
  pub fn forbidden(msg: impl Into<String>) -> Self;      // 403
  pub fn server_error(msg: impl Into<String>) -> Self;   // 500
  pub fn biz_error(code: i32, msg: impl Into<String>, data: Option<Value>) -> Self;
  pub fn internal(code: i32, msg: impl Into<String>, source: Option<Box<dyn Error + Send + Sync>>) -> Self;
  pub fn retry_limit(msg: impl Into<String>, retry_limit: u32) -> Self; // 1429
}

// From 实现（基础）
impl From<Error> for DataError;
impl From<std::io::Error> for DataError;
impl From<serde_json::Error> for DataError;
impl From<std::time::SystemTimeError> for DataError;
impl From<CtxError> for DataError;
impl From<std::net::AddrParseError> for DataError;

// From 实现（feature-gated）
#[cfg(feature = "with-uuid")]
impl From<uuid::Error> for DataError;

#[cfg(feature = "with-tokio")]
impl From<tokio::task::JoinError> for DataError;

#[cfg(feature = "with-config")]
impl From<config::ConfigError> for DataError;

// Result 类型别名
pub type DataResult<T> = core::result::Result<T, DataError>;
```

**使用示例：**
```rust
use hetus::common::DataError;

// 创建错误
let err = DataError::not_found("用户不存在");
let err = DataError::biz_error(1001, "余额不足", Some(json!({"balance": 100})));

// 在 Service 中使用
fn get_user(id: i64) -> Result<User, DataError> {
    user_bmc.get_by_id(&mm, id)
        .await?
        .ok_or_else(|| DataError::not_found("用户不存在"))
}
```

### Ctx - 请求上下文

```rust
pub struct Ctx(Arc<CtxInner>);  // clone 成本很低
pub struct CtxPayload(Map<String, Value>);
```

**创建上下文：**
```rust
use hetus::common::ctx::{Ctx, CtxPayload};
use hetus::common::time::now_offset;

// 构建载荷
let mut payload = CtxPayload::default();
payload.set_subject("user_123");      // 用户 ID
payload.set_tenant_id(456);           // 租户 ID
payload.set_expires_at(expire_time);  // 过期时间
payload.set("custom_key", json!("value"));  // 自定义数据

// 创建 Ctx
let ctx = Ctx::try_new(payload, Some(now_offset()), Some("req_id".to_string()))?;

// 便捷方法
let ctx = Ctx::new_root();           // 根用户上下文
let ctx = Ctx::new_super_admin();    // 超级管理员上下文
```

**获取信息：**
```rust
let user_id: i64 = ctx.user_id();
let tenant_id: i64 = ctx.tenant_id();
let req_time: &DateTime<FixedOffset> = ctx.req_time();
let request_id: Option<&str> = ctx.request_id();
let expires_at: Option<DateTime<FixedOffset>> = ctx.expires_at();

// 检查权限
if ctx.is_root() { /* ... */ }
```

### Time - 时间处理

```rust
use hetus::common::time::*;

// 获取当前时间
let utc_now: UtcDateTime = now_utc();      // UTC 时间
let offset_now: OffsetDateTime = now_offset();  // 本地时区时间
let local_now: OffsetDateTime = now();     // now_offset() 别名

// 时间戳
let millis: i64 = now_epoch_millis();
let seconds: i64 = now_epoch_seconds();

// 转换
let local: OffsetDateTime = to_local(utc_now);
let formatted: String = format_time(utc_now)?;

// 类型别名
pub type OffsetDateTime = DateTime<FixedOffset>;
pub type UtcDateTime = DateTime<Utc>;
pub type LocalDateTime = DateTime<Local>;
```

### Result Types - 响应包装

```rust
// 通用结果
pub struct WrapperResult<T> { pub data: T; }

// ID 结果
pub struct IdResult { pub id: serde_json::Value; }
pub struct IdI64Result { pub id: i64; }
pub struct IdStringResult { pub id: String; }
pub struct IdUuidResult { pub id: uuid::Uuid; }  // feature: with-uuid

// 使用
ok_json!(IdI64Result::from(id))
ok_json!(WrapperResult { data: users })
```

### SensitiveString - 敏感数据

```rust
use hetus::common::model::SensitiveString;

let secret = SensitiveString::new("password123");
// Display 和 Debug 会隐藏实际内容
println!("{}", secret);  // 输出: [REDACTED]
```

### Error - 错误类型

```rust
use hetus::common::error::{Error, Result};

pub enum Error {
    FailToB64uDecode(String),
    DateFailParse(String),
    KeyFail,
    PwdNotMatching,
    MissingEnv(String),
    WrongFormat(String),
    // ...
}

// 使用
let result: Result<T> = Err(Error::PwdNotMatching);
```

## ahash - 高性能 HashMap

```rust
use hetus::common::ahash::{HashMap, HashSet};

// 比 std::collections::HashMap 快 2-3 倍
let mut map: HashMap<String, i32> = HashMap::new();
map.insert("key".to_string(), 42);
```

## Serde Helpers

```rust
use hetus::common::serde::{deser_default_true, deser_default_false};

#[derive(Deserialize)]
struct MyConfig {
    #[serde(default = "deser_default_true")]
    enabled: bool,

    #[serde(default = "deser_default_false")]
    debug: bool,
}
```

## Best Practices

1. **Ctx 传递**: 始终通过 `mm.with_ctx(ctx)` 传递上下文给 ModelManager
2. **时间处理**: 使用 `now_offset()` 获取本地时区时间用于显示，`now_utc()` 用于存储
3. **HashMap**: 使用 `ahash::HashMap` 替代 `std::collections::HashMap`
4. **敏感数据**: 使用 `SensitiveString` 包装密码等敏感信息

## Examples from Codebase

- `crates/hetu-common/src/ctx/mod.rs` - Ctx 实现
- `crates/hetu-common/src/time/mod.rs` - 时间工具
- `jieyuan/jieyuan-server/src/access_control/auth_svc.rs` - Ctx 使用示例
