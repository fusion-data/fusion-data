# fusion-common LLM 使用指南

> fusion-common 是 Fusion-Data 项目的通用工具库，提供错误处理、上下文管理、加密、UUID 等基础功能。

## 架构概览

```
fusion-common/
├── ctx/          # 会话上下文 (Ctx, CtxPayload)
├── digest/       # 哈希与加密 (SHA256, HMAC, Base64Url)
├── error/        # 错误类型 (Error, DataError trait)
├── helper/       # 默认值辅助函数
├── model/        # 数据模型 (Result, SensitiveString)
├── serde/        # serde 工具
├── string/       # 字符串工具
├── time/         # 时间处理 (chrono 封装)
├── uuid/         # UUID 工具 (特征 Trait)
└── env/          # 环境变量工具
```

---

## 核心类型

### Error 枚举

```rust
pub enum Error {
  FailToB64uDecode(String),      // Base64Url 解码失败
  DateFailParse(String),          // 日期解析失败
  KeyFail,                        // 密钥错误
  PwdNotMatching,                 // 密码不匹配
  MissingEnv(String),             // 缺少环境变量
  WrongFormat(String),            // 格式错误
  FailedToSetEnv(..),             // 设置环境变量失败
  FailedToRemoveEnv(..),          // 删除环境变量失败
}
```

### DataError Trait

```rust
pub trait DataError: Error + Debug + Display + Serialize {
  fn code(&self) -> i32;
  fn msg(&self) -> &str;
  fn data(&self) -> Option<&serde_json::Value>;
  fn source(&self) -> Option<&(dyn Error + 'static)>;
}
```

### Result 类型

```rust
pub type Result<T> = core::result::Result<T, Error>;
```

---

## Ctx (会话上下文)

### CtxPayload

```rust
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CtxPayload(Map<String, Value>);

// 设置
payload.set_subject("user_id");
payload.set_expires_at(now_utc() + Duration::hours(24));
payload.set_i64("tenant_id", 1);
payload.set_string("role", "admin");
payload.set_bool("is_super_admin", false);
payload.set_strings("permissions", vec!["read", "write"]);

// 获取
let subject: Option<&str> = payload.get_subject();
let tenant_id: Option<i64> = payload.get_i64("tenant_id");
let permissions: Option<Vec<&str>> = payload.get_strings("permissions");
```

### Ctx

```rust
#[derive(Clone, Debug)]
pub struct Ctx(Arc<CtxInner>);

impl Ctx {
  pub const SUB: &str = "sub";          // 用户 ID
  pub const EXP: &str = "exp";          // 过期时间
  pub const TENANT_ID: &str = "tenant_id"; // 租户 ID

  // 创建
  pub fn try_new(payload: CtxPayload, req_time, req_id) -> Result<Self, CtxError>;
  pub fn new_root() -> Self;             // 根上下文 (sub="0")
  pub fn new_super_admin() -> Self;      // 超级管理员 (sub="1")

  // 获取
  pub fn get_user_id(&self) -> Option<i64>;
  pub fn user_id(&self) -> i64;          // 默认 0
  pub fn get_tenant_id(&self) -> Option<i64>;
  pub fn tenant_id(&self) -> i64;        // 默认 0
  pub fn req_time(&self) -> &DateTime<FixedOffset>;
  pub fn req_id(&self) -> &str;
  pub fn expires_at(&self) -> Option<DateTime<Utc>>;
  pub fn payload(&self) -> &CtxPayload;
}
```

### 使用示例

```rust
use fusions::common::ctx::{Ctx, CtxPayload};

let mut payload = CtxPayload::default();
payload.set_subject("12345");
payload.set_i64("tenant_id", 1);

let ctx = Ctx::try_new(payload, None, None)?;
let user_id = ctx.user_id();           // 12345
let tenant_id = ctx.tenant_id();       // 1
```

---

## 摘要与加密

### SHA256

```rust
use fusions::common::digest::{sha256, sha256_string};

// 二进制
let hash = sha256(b"data");

// 十六进制字符串
let hash_str = sha256_string(b"data");
```

### HMAC-SHA256

```rust
use fusions::common::digest::{hmac_sha256, hmac_sha256_string};

let mac = hmac_sha256(b"secret", b"data")?;
let mac_str = hmac_sha256_string(b"secret", b"data")?;
```

### Base64Url

```rust
use fusions::common::digest::{b64u_encode, b64u_decode, b64u_decode_to_string};

let encoded = b64u_encode(b"hello world");
let decoded = b64u_decode(&encoded)?;
let decoded_str = b64u_decode_to_string(&encoded)?;
```

---

## UUID 工具

### ExtraUuid

```rust
use fusions::common::uuid::ExtraUuid;

let uuid = ExtraUuid::new();
let short_id = uuid.short_id();           // 12字符短 ID
let urn = uuid.to_urn();                  // urn:uuid:...
let string = uuid.to_string();
```

### ExtraBase64

```rust
use fusions::common::uuid::ExtraBase64;

let uuid = ExtraBase64::new();
let b64 = uuid.to_b64();                  // Base64 编码
let short_b64 = uuid.to_short_b64();      // 22字符
```

---

## 时间处理

### 时间类型别名

```rust
pub type OffsetDateTime = DateTime<FixedOffset>;
pub type UtcDateTime = DateTime<Utc>;
pub type LocalDateTime = DateTime<Local>;
```

### 时间函数

```rust
use fusions::common::time::{now, now_utc, now_offset, now_epoch_millis, now_epoch_seconds};

let now = now();                   // 本地时间
let utc = now_utc();              // UTC 时间
let millis = now_epoch_millis();  // 毫秒时间戳
let secs = now_epoch_seconds();   // 秒时间戳
```

### 时间转换

```rust
use fusions::common::time::{parse_utc, format_time, utc_from_millis, datetime_from_millis};

let dt = parse_utc("2024-01-01T00:00:00Z")?;
let formatted = format_time(dt)?;
let utc = utc_from_millis(1704067200000);
let local = datetime_from_millis(1704067200000);
```

### chrono 导出

```rust
use fusions::common::time::{DateTime, Duration, FixedOffset, Local, Utc, OffsetDateTime};
```

---

## 敏感数据处理

### SensitiveString

```rust
use fusions::common::model::sensitive::SensitiveString;

let secret = SensitiveString::new("my-secret-key");
let masked = secret.mask();           // "********"
```

### UriString

```rust
use fusions::common::model::sensitive::UriString;

let uri = UriString::new("https://api.example.com");
let sanitized = uri.sanitize();       // 移除敏感信息
```

---

## 辅助函数

### 默认值

```rust
use fusions::common::helper::{default_bool_true, default_bool_false, default_i64_0, default_i64_1};

#[derive(Default)]
struct Config {
  #[serde(default = "helper::default_bool_true")]
  enabled: bool,
  #[serde(default = "helper::default_i64_0")]
  retry_count: i64,
}
```

---

**版本**: fusion-common v0.1.0+
