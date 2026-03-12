# hetu-core

核心框架：Application 生命周期、Component 依赖注入、Configuration 配置、Plugin 插件系统。

## Imports

```rust
use hetus::core::{
    Application,
    application::{ApplicationBuilder, ShutdownRecv},
    component::{Component, ComponentArc, ComponentInstaller},
    configuration::{Configurable, ConfigRegistry, HetuSetting, SecuritySetting},
    plugin::Plugin,
    Result,  // Result<T> = Result<T, hetu_common::DataError>
};
use hetus::core::async_trait;
use hetus::DataError;  // 或 use hetu_common::DataError;
```

## Application 生命周期

### 完整示例

```rust
use hetus::core::{Application, plugin::Plugin, async_trait, Result};
use hetus::db::DbPlugin;

pub struct MyPlugin {
    config: MyConfig,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["hetu_db::DbPlugin"]  // 确保 DbPlugin 先加载
    }

    fn immediately(&self) -> bool {
        false  // 异步构建
    }

    fn immediately_build(&self, app: &mut ApplicationBuilder) {
        // 同步阶段：添加配置源
        app.add_config_source(File::from_str(CONFIG, FileFormat::Toml));
    }

    async fn build(&self, app: &mut ApplicationBuilder) {
        // 异步阶段：获取配置和组件
        let config: MyConfig = app.get_config()?;
        let mm: ModelManager = app.component();

        // 创建并注册组件
        let service = MyService::new(mm, config);
        app.add_component(service);
    }
}

#[tokio::main]
async fn main() -> Result<(), hetus::DataError> {
    // 构建并运行应用
    let app = Application::builder()
        .add_config_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
        .add_plugin(DbPlugin)
        .add_plugin(MyPlugin::default())
        .run()
        .await?;

    // 获取组件
    let service: MyService = app.component();

    // 等待关闭信号
    Application::await_shutdown().await;
    Ok(())
}
```

### 信号处理

```rust
// 获取 shutdown receiver
let shutdown_rx = app.shutdown_recv().await;

// 在服务器中使用
WebServerBuilder::new(router)
    .with_shutdown(shutdown_rx)
    .build()
    .await?;

// 手动触发关闭
Application::shutdown().await;

// 等待关闭完成
let closed = Application::await_shutdown().await;
```

## Component 依赖注入

### 使用 derive 宏

```rust
use hetus_core_macros::Component;
use hetus::db::ModelManager;

#[derive(Clone, Component)]
pub struct UserService {
    #[component]  // 自动注入
    mm: ModelManager,

    #[component]
    cache: CacheService,

    config: Arc<UserConfig>,  // 非注入字段
}

impl UserService {
    pub fn new(mm: ModelManager, cache: CacheService, config: Arc<UserConfig>) -> Self {
        Self { mm, cache, config }
    }
}
```

### 注册组件

```rust
use hetus::core::component::{ComponentInstaller, submit};

// 定义 Installer
pub struct UserServiceInstaller;

impl ComponentInstaller for UserServiceInstaller {
    fn install(&self, app: &ApplicationBuilder) -> Result<()> {
        let mm: ModelManager = app.component()?;
        let cache: CacheService = app.component()?;
        let config: Arc<UserConfig> = app.get_config()?;

        let service = UserService::new(mm, cache, config);
        app.add_component(service);
        Ok(())
    }
}

// 使用 inventory 注册
submit! {
    &UserServiceInstaller as &dyn ComponentInstaller
}
```

### 获取组件

```rust
// 在 Plugin 中
let service: UserService = app.component();

// 在 Application 中
let service: UserService = app.component();

// 可选获取
let service: Option<UserService> = app.component_opt();
```

## Configuration 配置系统

### 定义配置

```rust
use hetus::core::configuration::Configurable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: Duration,
}

fn default_idle_timeout() -> Duration {
    Duration::from_secs(600)
}

impl Configurable for DatabaseConfig {
    fn config_prefix() -> &'static str {
        "myapp.database"
    }
}
```

### 配置源

```rust
use hetus::core::configuration::{File, FileFormat};

// TOML 文件
app.add_config_source(File::from_path("config.toml", FileFormat::Toml)?);

// 嵌入式配置
app.add_config_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml));

// 环境变量（自动支持）
// MYAPP__DATABASE__URL=postgres://...
```

### 获取配置

```rust
// 使用 config_prefix
let config: DatabaseConfig = app.get_config()?;

// 指定路径
let config: DatabaseConfig = app.get_config_by_path("myapp.database")?;

// 内置配置
let settings: HetuSetting = app.hetu_setting();
let security: SecuritySetting = settings.security();
```

### 配置格式 (TOML)

```toml
[myapp.database]
url = "postgresql://user:pass@localhost:5432/mydb"
max_connections = 10
idle_timeout = "10s"

[myapp.server]
host = "0.0.0.0"
port = 8080
```

## Plugin 插件系统

### Plugin Trait

```rust
#[async_trait]
pub trait Plugin: Any + Send + Sync {
    /// 插件名称，用于依赖解析
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 依赖的其他插件
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// 是否立即执行（同步阶段）
    fn immediately(&self) -> bool {
        false
    }

    /// 同步构建阶段
    fn immediately_build(&self, _app: &mut ApplicationBuilder) {}

    /// 异步构建阶段
    async fn build(&self, _app: &mut ApplicationBuilder) {}
}
```

### 插件顺序

```rust
// 依赖会自动排序
// DbPlugin 先加载，MyPlugin 后加载
Application::builder()
    .add_plugin(MyPlugin)      // 声明依赖 DbPlugin
    .add_plugin(DbPlugin)
    .run()
    .await?;
```

## DataError 错误处理

> **重要**: `DataError` 定义在 `hetu-common` 中，hetu-core 不重新导出。
> 使用 `hetu_common::DataError` 或 `hetus::DataError`。

```rust
// 推荐方式
use hetu_common::DataError;

// 或通过 hetus meta-crate
use hetus::DataError;

// 创建错误
DataError::bad_request("Invalid input")      // 400
DataError::unauthorized("Token expired")     // 401
DataError::forbidden("Access denied")        // 403
DataError::not_found("User not found")       // 404
DataError::conflicted("Resource exists")     // 409
DataError::server_error("Internal error")    // 500

// 业务错误
DataError::biz_error(
    1001,  // 业务错误码
    "User quota exceeded",
    Some(json!({ "limit": 100, "current": 105 }))
)
```

### 本地错误转换

hetu-core 为本地错误类型提供了 From 实现：

```rust
// security::Error -> DataError (在 security/error.rs 中)
impl From<Error> for hetu_common::DataError {
  fn from(err: Error) -> Self {
    match err {
      Error::TokenExpired => hetu_common::DataError::unauthorized("Token expired"),
      Error::SignatureNotMatching => hetu_common::DataError::unauthorized("Signature not matching"),
      Error::InvalidPassword => hetu_common::DataError::unauthorized("Invalid password"),
      _ => hetu_common::DataError::server_error(err.to_string()),
    }
  }
}

// ConfigureError -> DataError (在 configuration/error.rs 中)
impl From<ConfigureError> for hetu_common::DataError {
  fn from(err: ConfigureError) -> Self {
    hetu_common::DataError::server_error(err.to_string())
  }
}

// ComponentError -> DataError (在 component/error.rs 中)
impl From<ComponentError> for hetu_common::DataError {
  fn from(err: ComponentError) -> Self {
    hetu_common::DataError::internal(500, err.to_string(), Some(Box::new(err)))
  }
}
```

### 使用示例

```rust
use hetu_common::DataError;

// 在 Service 中使用
fn get_user(id: i64) -> Result<User, DataError> {
    user_bmc.get_by_id(&mm, id)
        .await?  // SqlError 自动转换为 DataError
        .ok_or_else(|| DataError::not_found("用户不存在"))
}
```

## Best Practices

1. **Plugin 顺序**: 使用 `dependencies()` 声明依赖，确保加载顺序正确
2. **Component 注入**: 使用 `#[derive(Component)]` 简化依赖注入
3. **配置分层**: TOML 默认配置 + 环境变量覆盖
4. **错误转换**: 为自定义错误实现 `From<DataError>` 以使用 `?`
5. **优雅关闭**: 使用 `shutdown_recv()` 实现优雅关闭

## Examples from Codebase

- `crates/hetu-core/src/application.rs` - Application 实现
- `crates/hetu-core/src/component/mod.rs` - Component 系统
- `crates/hetu-db/src/lib.rs` - DbPlugin 示例
- `hetuflow/hetuflow-server/src/main.rs` - 完整应用示例
