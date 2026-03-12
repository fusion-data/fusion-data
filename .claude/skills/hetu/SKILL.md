---
name: hetu
description: Hetu core library patterns, hetu-common, hetu-core, hetu-db, hetu-web, hetu-ai, hetusql, hetus, Component, Application, ModelManager, Ctx, FlowNode, BMC pattern
globs:
  - 'crates/hetu-*/**/*.rs'
  - 'crates/hetus/**/*.rs'
  - 'crates/hetusql/**/*.rs'
---

# Hetu Core Library

## Quick Reference

| Module   | Import                    | Key Types                                                |
| -------- | ------------------------- | -------------------------------------------------------- |
| Common   | `use hetus::common::*;`   | `Ctx`, `CtxPayload`, `DataError`, `OffsetDateTime`, `SensitiveString` |
| Core     | `use hetus::core::*;`     | `Application`, `Plugin`, `Component`                     |
| DB       | `use hetus::db::*;`       | `ModelManager`, `DbPlugin`                               |
| Web      | `use hetus::web::*;`      | `Router`, `WebError`, `WebResult`, `WebServerBuilder`    |
| AI       | `use hetus::ai::*;`       | `ClientFactory`, `FlowRunner`, `Graph`, `Task`           |
| SQL      | `use hetus::sql::*;`      | `Fields`, `FilterNodes`, `Page`, `DbConfig`              |
| Security | `use hetus::security::*;` | `make_token`, `OAuthClient`                              |

## Core Patterns

### Application Setup

```rust
use hetus::core::{Application, plugin::Plugin, async_trait, Result};
use hetus::DataError;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my_plugin" }
    fn dependencies(&self) -> Vec<&str> { vec!["hetu_db::DbPlugin"] }

    async fn build(&self, app: &mut ApplicationBuilder) {
        let mm: ModelManager = app.component();
        // ... build logic
    }
}

#[tokio::main]
async fn main() -> Result<(), DataError> {
    let app = Application::builder()
        .add_plugin(DbPlugin)
        .add_plugin(MyPlugin)
        .run()
        .await?;

    Application::await_shutdown().await;
    Ok(())
}
```

### Component with DI

```rust
use hetus_core_macros::Component;

#[derive(Clone, Component)]
pub struct MyService {
    #[component]
    mm: ModelManager,
    config: Arc<MyConfig>,
}
```

### Context Pattern

```rust
use hetus::common::ctx::{Ctx, CtxPayload};

let mut payload = CtxPayload::default();
payload.set_subject("user_123");
payload.set_tenant_id(456);

let ctx = Ctx::try_new(payload, Some(now_offset()), None)?;
let user_id: i64 = ctx.user_id();
```

### Error Handling

> **注意**: `DataError` 定义在 `hetu-common` 中。

```rust
use hetus::common::DataError;  // 或 use hetus::DataError;

// 便捷方法
DataError::bad_request("Invalid input")       // 400
DataError::not_found("User not found")        // 404
DataError::unauthorized("Token expired")      // 401
DataError::forbidden("Access denied")         // 403
DataError::conflicted("Resource exists")      // 409
DataError::server_error("Internal error")     // 500
DataError::biz_error(1001, "Business error", Some(json!({"key": "value"})))
```

### Web Handler

```rust
use hetus::web::{WebResult, WebError, ok_json};
use axum::Json;

async fn get_user(id: Path<i64>, mm: ModelManager) -> WebResult<User> {
    let user = UserBmc::get(&mm, *id).await?
        .ok_or_else(|| WebError::not_found("User not found"))?;
    ok_json!(user)
}
```

### BMC Pattern

```rust
use hetusql::{base::DbBmc, BmcConfig, crud_fns};

pub struct UserBmc;

impl DbBmc for UserBmc {
    fn _bmc_config() -> &'static BmcConfig {
        &BmcConfig::new_table("users")
            .with_column_id("id")
            .with_id_generated_by_db(true)
    }
}

// CRUD helpers
crud_fns::create::<UserBmc, _>(&mm, user).await?;
crud_fns::get_by_id::<UserBmc, User, _>(&mm, Id::I64(id)).await?;
crud_fns::list::<UserBmc, User, _>(&mm, filter, page).await?;
```

### Transaction

```rust
// 推荐：闭包式事务
mm.transaction(|mm| async move {
    UserBmc::create(&mm, user).await?;
    ProfileBmc::create(&mm, profile).await?;
    Ok(())
}).await?;

// 手动事务
let mm_txn = mm.txn_cloned();
mm_txn.dbx().begin_txn().await?;
// ... 业务逻辑
mm_txn.dbx().commit_txn().await?;
```

## Feature Flags (hetus)

| Feature        | Description                |
| -------------- | -------------------------- |
| `web`          | Axum web framework         |
| `db`           | PostgreSQL + ModelManager  |
| `db-sqlite`    | SQLite support             |
| `security`     | JWT + OAuth2               |
| `ai`           | LLM providers + graph_flow |
| `grpc`         | Tonic gRPC                 |
| `openapi`      | utoipa documentation       |
| `logforth`     | Logging framework          |
| `tracing`      | Distributed tracing        |
| `full`         | All features               |
| `api`          | web + db + security        |
| `microservice` | api + grpc                 |

## Common Mistakes

| Mistake                              | Correct                                 |
| ------------------------------------ | --------------------------------------- |
| `use hetus::core::DataError`         | Use `use hetus::DataError` or `use hetus::common::DataError` |
| `ctx.user_id()` without tenant check | Use `ctx.tenant_id()` for multi-tenancy |
| Manual transaction management        | Use `mm.transaction()` closure          |
| `Rc` in Component                    | Use `Arc` for thread-safe sharing       |
| Direct `sqlx` queries                | Use BMC pattern for consistency         |
| Missing `?` on Result                | Propagate errors with `?`               |

## References (按需加载)

- [hetu-common](references/hetu-common.md) - ctx, time, uuid, error, serde, ahash
- [hetu-core](references/hetu-core.md) - Application, Component, Configuration, Plugin, signal
- [hetu-db](references/hetu-db.md) - ModelManager, DbPlugin, store
- [hetu-web](references/hetu-web.md) - Router, middleware, extract, error
- [hetu-ai](references/hetu-ai.md) - LLM providers, agents, graph_flow, embeddings
- [hetu-security](references/hetu-security.md) - JWT, OAuth2
- [hetusql](references/hetusql.md) - Entity, Fields, FilterNodes, BMC, pagination, transactions
- [hetus](references/hetus.md) - Feature flags, re-exports

## Related Skills

- `sql-database`: ORM patterns, BMC/Service layers
- `rust-backend`: Component patterns, service integration
- `tokio-axum-patterns`: Tokio/Axum best practices
