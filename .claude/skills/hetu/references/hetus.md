# hetus

聚合包：统一入口、Feature Flags 组合。

## Cargo.toml 配置

```toml
[dependencies]
hetus = { version = "0.1", features = ["full"] }

# 或按需选择
hetus = { version = "0.1", features = ["web", "db", "security"] }

# 微服务
hetus = { version = "0.1", features = ["microservice"] }
```

## Feature Flags

### 功能模块

| Feature     | 描述                       | 依赖                                 |
| ----------- | -------------------------- | ------------------------------------ |
| `web`       | Axum Web 框架              | hetu-web, axum, tower-http           |
| `db`        | PostgreSQL + ModelManager  | hetu-db, hetusql, sqlx/postgres      |
| `db-sqlite` | SQLite 支持                | hetu-db, sqlx/sqlite                 |
| `security`  | JWT + OAuth2               | hetu-security, oauth2, openidconnect |
| `ai`        | LLM providers + graph_flow | hetu-ai, rig-core                    |
| `grpc`      | Tonic gRPC                 | hetu-grpc, tonic                     |

### 可选功能

| Feature    | 描述            |
| ---------- | --------------- |
| `openapi`  | utoipa API 文档 |
| `logforth` | 日志框架        |
| `tracing`  | 分布式追踪      |
| `ulid`     | ULID 支持       |

### 便捷组合

| Feature        | 包含                            |
| -------------- | ------------------------------- |
| `full`         | web + db + security + ai + grpc |
| `api`          | web + db + security             |
| `web-server`   | web + db                        |
| `microservice` | api + grpc                      |

## Re-export 结构

```rust
// 核心（始终可用）
use hetus::{
    common,   // hetu_common
    core,      // hetu_core
    macros,    // hetu_core_macros
};

// 条件导出
#[cfg(feature = "ai")]
use hetus::ai;

#[cfg(feature = "db")]
use hetus::{db, sql};

#[cfg(feature = "security")]
use hetus::security;

#[cfg(feature = "web")]
use hetus::web;

#[cfg(feature = "grpc")]
use hetus::grpc;
```

## 快速启动模板

### Web API 服务

```rust
// Cargo.toml
// [dependencies]
// hetus = { version = "0.1", features = ["api"] }

use hetus::{
    core::{Application, plugin::Plugin, async_trait},
    common::ctx::{Ctx, CtxPayload},
    db::DbPlugin,
    web::{Router, WebServerBuilder, WebError, WebResult},
    sql::ModelManager,
};

pub struct ApiPlugin;

#[async_trait]
impl Plugin for ApiPlugin {
    fn dependencies(&self) -> Vec<&str> {
        vec!["hetu_db::DbPlugin"]
    }

    async fn build(&self, app: &mut ApplicationBuilder) {
        let mm: ModelManager = app.component();

        let router = Router::new()
            .route("/api/users", get(list_users))
            .route("/api/users/:id", get(get_user))
            .with_state(mm);

        app.add_component(router);
    }
}

#[tokio::main]
async fn main() -> hetus::core::Result<()> {
    let app = Application::builder()
        .add_plugin(DbPlugin)
        .add_plugin(ApiPlugin)
        .run()
        .await?;

    let router: Router = app.component();

    WebServerBuilder::new(router)
        .with_shutdown(app.shutdown_recv().await)
        .build()
        .await?;

    Application::await_shutdown().await;
    Ok(())
}
```

### AI 服务

```rust
// Cargo.toml
// [dependencies]
// hetus = { version = "0.1", features = ["ai", "web"] }

use hetus::{
    core::Application,
    ai::{ClientFactory, AgentConfig, graph_flow::*},
    web::{Router, WebServerBuilder},
};

async fn chat_handler(
    Json(req): Json<ChatRequest>,
) -> WebResult<ChatResponse> {
    let factory = ClientFactory::new();
    let client = factory.openai(&req.api_key)?;
    let agent = factory.openai_agent(&req.config, &client)?;

    let response = agent.invoke(&req.message, vec![]).await?;
    ok_json!(ChatResponse::from(response))
}
```

### 微服务

```rust
// Cargo.toml
// [dependencies]
// hetus = { version = "0.1", features = ["microservice"] }

use hetus::{
    core::Application,
    db::DbPlugin,
    grpc::GrpcPlugin,
    security::SecurityPlugin,
};

#[tokio::main]
async fn main() -> hetus::core::Result<()> {
    let app = Application::builder()
        .add_plugin(DbPlugin)
        .add_plugin(SecurityPlugin)
        .add_plugin(GrpcPlugin)
        .add_plugin(MyServicePlugin)
        .run()
        .await?;

    Application::await_shutdown().await;
    Ok(())
}
```

## 默认 Features

```toml
[features]
default = ["common-default"]
common-default = ["hetu-common/with-uuid"]
```

## 常用导入汇总

```rust
// 核心
use hetus::core::{Application, Result};
use hetus::common::ctx::{Ctx, DataError, CtxPayload};
use hetus::common::time::{now_offset, OffsetDateTime};

// Web
use hetus::web::{Router, WebError, WebResult, WebServerBuilder, ok_json};
use axum::{Json, Path, State, routing::{get, post}};

// 数据库
use hetus::sql::{ModelManager, Fields, FilterNodes, Page, PageResult};
use hetusql::{DbBmc, BmcConfig, crud_fns};

// AI
use hetus::ai::{ClientFactory, AgentConfig, Graph, FlowRunner};

// 安全
use hetus::security::{make_token, validate_token, OAuthClient};
```

## Best Practices

1. **按需引入**: 只启用需要的 features，减少编译时间和二进制大小
2. **使用组合**: 使用 `api`、`microservice` 等预定义组合
3. **版本一致**: 确保 hetus 与各个子模块版本一致

## Examples from Codebase

- `crates/hetus/src/lib.rs` - 聚合包实现
- `hetuflow/hetuflow-server/src/main.rs` - 完整服务示例
- `hetumind/hetumind-studio/src/main.rs` - AI 服务示例
