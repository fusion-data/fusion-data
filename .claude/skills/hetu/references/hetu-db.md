# hetu-db

数据库抽象层：ModelManager、DbPlugin、PostgreSQL/SQLite 支持。

## Imports

```rust
use hetus::db::{DbPlugin, ModelManager};
use hetusql::{DbConfig, DbxProvider};
```

## DbPlugin

### 在 Application 中使用

```rust
use hetus::db::DbPlugin;
use hetus::core::{Application, plugin::Plugin};

#[tokio::main]
async fn main() -> hetus::core::Result<()> {
    let app = Application::builder()
        .add_plugin(DbPlugin)  // 自动加载配置
        .add_plugin(MyPlugin)
        .run()
        .await?;

    // 获取 ModelManager
    let mm: ModelManager = app.component();
    Ok(())
}
```

### 配置 (TOML)

```toml
[hetu.db]
enable = true
url = "postgresql://user:pass@localhost:5432/mydb"
max_connections = 10
idle_timeout = "10s"
acquire_timeout = "5s"
schema_search_path = "my_schema"
application_name = "my_app"
```

## DbConfig

### 完整配置

```rust
use hetusql::{DbConfig, DbxProvider};
use std::time::Duration;
use std::borrow::Cow;

let config = DbConfig {
    enable: true,

    // 方式1：完整 URL
    url: Some("postgresql://user:pass@localhost:5432/mydb".into()),

    // 方式2：分开配置
    provider: Some(DbxProvider::Postgres),
    host: Some("localhost".to_string()),
    port: Some(5432),
    database: Some("mydb".to_string()),
    username: Some("user".to_string()),
    password: Some("pass".to_string()),

    // 连接池
    max_connections: Some(10),
    min_connections: Some(2),
    idle_timeout: Some(Duration::from_secs(600)),
    acquire_timeout: Some(Duration::from_secs(30)),
    max_lifetime: Some(Duration::from_secs(3600)),

    // 日志
    sqlx_logging: Some(true),
    sqlx_logging_level: Some("debug".into()),

    // PostgreSQL 专用
    schema_search_path: Some("my_schema".into()),
    application_name: Some("my_app".into()),

    // SQLite 专用
    sqlcipher_key: Some(Cow::Borrowed("encryption_key")),
};
```

### SQLite 配置

```toml
[hetu.db]
enable = true
url = "sqlite:data.db?mode=rwc"
# 或
provider = "sqlite"
database = "data.db"
```

## ModelManager

### 创建

```rust
use hetusql::ModelManager;

// 从配置创建
let mm = ModelManager::new(&db_config, Some("my_app")).await?;

// 从 Application 获取
let mm: ModelManager = app.component();
```

### 设置上下文

```rust
use hetus::common::ctx::Ctx;

// 克隆并设置上下文
let mm = mm.with_ctx(ctx);

// 上下文会自动传递给 BMC 操作
```

### 过滤器拦截器 (多租户)

```rust
// 自动添加租户过滤条件
let mm = mm.with_filter_interceptor(|bmc_config, ctx, filters| {
    if bmc_config.has_owner_id {
        // 修改 filters 添加 tenant_id 条件
    }
    Ok(filters)
});
```

### 事务 API

```rust
// 方式1：闭包式事务（推荐）
mm.transaction(|mm| async move {
    UserBmc::create(&mm, user).await?;
    ProfileBmc::create(&mm, profile).await?;
    Ok::<_, DataError>(())
}).await?;

// 方式2：手动事务
let mm_txn = mm.txn_cloned();
mm_txn.dbx().begin_txn().await?;

UserBmc::create(&mm_txn, user).await?;

mm_txn.dbx().commit_txn().await?;
// 或
mm_txn.dbx().rollback_txn().await?;
```

### 获取底层连接

```rust
// Dbx 抽象
let dbx = mm.dbx();

// 原始连接（谨慎使用）
let pool = mm.db();
```

## Dbx 抽象

```rust
use hetusql::store::{Dbx, DbxPostgres, DbxSqlite, create_dbx};

pub enum Dbx {
    Postgres(DbxPostgres),
    Sqlite(DbxSqlite),
}

// 创建
let dbx = create_dbx(&db_config, Some("my_app")).await?;

// 事务
dbx.begin_txn().await?;
dbx.commit_txn().await?;
dbx.rollback_txn().await?;
```

## Best Practices

1. **使用 DbPlugin**: 在 Application 中使用 DbPlugin 自动管理生命周期
2. **传递 Ctx**: 始终通过 `mm.with_ctx(ctx)` 传递用户上下文
3. **事务**: 使用闭包式事务 `mm.transaction()` 自动管理提交/回滚
4. **多租户**: 使用 `with_filter_interceptor` 自动添加租户过滤
5. **连接池**: 合理配置 `max_connections` 和 `idle_timeout`

## Examples from Codebase

- `crates/hetu-db/src/lib.rs` - DbPlugin 实现
- `crates/hetusql/src/model_manager.rs` - ModelManager 实现
- `jieyuan/jieyuan-server/src/user/user_svc.rs` - 事务使用示例
