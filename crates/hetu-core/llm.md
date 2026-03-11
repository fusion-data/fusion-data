# hetu-core LLM 使用指南

> hetu-core 是 Fusion-Data 项目的核心库，提供 Application 容器、依赖注入、配置管理、安全工具等核心功能。

## 架构概览

```
hetu-core/
├── application/      # 应用容器与构建器
├── component/        # 组件系统 (依赖注入)
├── configuration/    # 配置管理 (Configurable, ConfigRegistry)
├── concurrent/       # 并发任务处理
├── file/             # 文件工具
├── logforth/         # 日志系统 (logforth)
├── plugin/           # 插件系统
├── security/         # 安全工具 (JWT, 密码)
├── signal/           # 信号处理
├── timer/            # 定时器
├── tracing/          # 分布式追踪
└── metas/            # 元信息
```

---

## Application 应用容器

### ApplicationBuilder

```rust
use hetu_core::{Application, ApplicationBuilder, DataError, Builder};

let mut builder = Application::builder();

// 添加配置源
builder.add_config_source(File::from_str(config_str, FileFormat::Toml));

// 添加插件
builder.add_plugin::<LogforthPlugin>(LogforthPlugin);
builder.add_plugin::<DbPlugin>(DbPlugin);

// 添加组件
builder.add_component(my_component);

// 构建应用
let app = builder.build().await?;
```

### Application

```rust
// 获取全局应用
let app = Application::global();

// 获取组件
let component: MyComponent = app.component();

// 获取配置
let config: MyConfig = app.get_config()?;

// 获取底层 Config
let config: Arc<Config> = app.underlying_config();

// 获取启动时间
let start_time = app.start_time();

// 获取配置注册表
let registry = app.config_registry();
```

---

## Component 组件系统

### Component Trait

```rust
use hetu_core::{component::Component, DataError, application::ApplicationBuilder};

#[derive(Clone)]
pub struct MyComponent {
  config: Arc<MyConfig>,
}

impl Component for MyComponent {
  fn build(app: &ApplicationBuilder) -> Result<Self> {
    let config = app.get_config::<MyConfig>()?;
    Ok(Self { config: Arc::new(config) })
  }
}

// 使用
let component: Arc<MyComponent> = app.get_component_arc()?;
let component: MyComponent = app.component();
```

### ComponentArc / DynComponentArc

```rust
use hetu_core::component::{ComponentArc, DynComponentArc};

// 类型安全的组件引用
let component_arc: ComponentArc<MyComponent> = app.get_component_arc()?;

// 动态组件引用 (用于运行时)
let dyn_component: DynComponentArc = app.get_component_ref_by_name("MyComponent")?;
let downcast: ComponentArc<MyComponent> = dyn_component.downcast()?;
```

### ComponentInstaller (自动注入)

```rust
use hetu_core::{component::ComponentInstaller, Component, application::ApplicationBuilder, submit_component};

pub struct MyComponentInstaller;

impl ComponentInstaller for MyComponentInstaller {
  fn dependencies(&self) -> Vec<&str> {
    vec!["hetu_core::configuration::AppSetting"]
  }

  fn install_component(&self, app: &mut ApplicationBuilder) -> Result<()> {
    let component = MyComponent::build(app)?;
    app.add_component(component);
    Ok(())
  }
}

submit_component!(MyComponentInstaller);
```

---

## Plugin 插件系统

### Plugin Trait

```rust
use hetu_core::{plugin::Plugin, application::ApplicationBuilder, async_trait};

#[derive(Clone)]
pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
  fn name(&self) -> &str {
    "my-plugin"
  }

  fn dependencies(&self) -> Vec<&str> {
    vec!["DbPlugin"]
  }

  async fn build(&self, app: &mut ApplicationBuilder) {
    // 初始化插件
    let config = app.get_config::<MyConfig>().unwrap();
    // ...
  }
}

// 添加插件
app.add_plugin(MyPlugin);
```

---

## Configuration 配置管理

### Configurable Trait

```rust
use hetu_core::configuration::Configurable;

#[derive(Debug, Clone, Deserialize)]
pub struct MyConfig {
  pub enabled: bool,
  pub host: String,
  pub port: u16,
}

impl Configurable for MyConfig {
  fn config_prefix() -> &'static str {
    "my_service"
  }
}

// 使用
let config: MyConfig = app.get_config()?;
```

### ConfigRegistry Trait

```rust
use hetu_core::configuration::ConfigRegistry;

pub trait ConfigRegistry {
  fn get_config<T>(&self) -> ConfigureResult<T>
  where
    T: DeserializeOwned + Configurable;

  fn get_config_by_path<T>(&self, path: &str) -> ConfigureResult<T>
  where
    T: DeserializeOwned;
}

// 获取任意路径的配置
let db_config: DbConfig = app.get_config_by_path("fusion.db")?;
```

### FusionSetting

```rust
use hetu_core::configuration::FusionSetting;

let hetu_setting = app.hetu_setting();
let app_setting = hetu_setting.app();
let security_setting = hetu_setting.security();
let log_setting = hetu_setting.log();
```

---

## DataError 错误处理

```rust
use hetu_core::DataError;

// 从其他错误转换
impl From<SqlError> for DataError {
  fn from(err: SqlError) -> Self {
    match err {
      SqlError::EntityNotFound => DataError::not_found("实体不存在"),
      SqlError::UniqueViolation => DataError::conflict("唯一性冲突"),
      _ => DataError::internal(500, "内部错误", Some(Box::new(err))),
    }
  }
}

// 使用
fn get_user(id: i64) -> Result<User, DataError> {
  user_bmc.get_by_id(&mm, id)
    .await
    .map_err(DataError::from)?
    .ok_or_else(|| DataError::not_found("用户不存在"))
}
```

---

## 安全工具

### JWT (HS256)

```rust
use hetu_core::security::jose::{encode_jwt_hs256, decode_jwt_hs256, JwtPayload, JwsHeader};

let mut payload = JwtPayload::new();
payload.set_subject("user_id");
payload.set_expires_at(&SystemTime::now() + Duration::hours(24));

let token = encode_jwt_hs256(secret_key, &payload)?;
let (decoded_payload, header): (JwtPayload, JwsHeader) = decode_jwt_hs256(secret_key, &token)?;
```

### JWT (ES256)

```rust
use hetu_core::security::jose::{encode_jwt_es256, decode_jwt_es256};

let token = encode_jwt_es256(private_key, &payload)?;
let (payload, header) = decode_jwt_es256(public_key, &token)?;
```

### JWE 加密

```rust
use hetu_core::security::jose::{encrypt_jwe_dir, decrypt_jwe_dir};

let encrypted = encrypt_jwe_dir(secret_key, &payload)?;
let (decrypted_payload, header) = decrypt_jwe_dir(secret_key, &encrypted)?;
```

### 密码处理 (Argon2)

```rust
use hetu_core::security::pwd::{generate_pwd, verify_pwd, is_strong_password};

// 生成密码
let hashed = generate_pwd("MyPassword123").await?;
// 格式: #1#$argon2i$v=19$m=4096,t=3,p=1$...

// 验证密码
let version = verify_pwd("MyPassword123", &hashed).await?;

// 校验密码强度
let is_strong = is_strong_password("MyPassword123"); // 至少8位，大小写+数字
```

### RECOMMENDED_LENGTH

```rust
use hetu_core::security::RECOMMENDED_LENGTH; // 16 字节
```

---

## 信号处理

```rust
use hetu_core::utils::wait_exit_signals;

wait_exit_signals().await; // 等待 SIGINT, SIGTERM
```

---

## 并发任务

### TaskServiceHandle

```rust
use hetu_core::concurrent::{ServiceTask, TaskServiceHandle, TaskResult};

let handle = tokio::spawn(async {
  // 任务逻辑
  TaskResult::empty()
});

let service_handle = ServiceHandle::new("my-service", handle);

// 等待完成
let result = handle.complete().await;
```

---

## 日志系统

### LogforthPlugin

```rust
use hetu_core::logforth::LogforthPlugin;

app.add_plugin(LogforthPlugin);
```

---

**版本**: hetu-core v0.1.0+
