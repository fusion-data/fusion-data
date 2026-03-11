# fusion-db LLM 使用指南

> fusion-db 是 Fusion-Data 项目的数据库层封装，提供 `ModelManager` 和 `DbPlugin`，集成 fusionsql ORM。

## 架构概览

```
fusion-db/
├── acs/           # 访问控制系统 (PBAC, 待实现)
├── lib.rs         # DbPlugin, ModelManager 导出
└── resources/
    └── default.toml
```

---

## DbPlugin 插件

### 使用方式

```rust
use ultimate_db::DbPlugin;
use ultimate_core::ApplicationBuilder;

let mut app = Application::builder();
app.add_plugin(DbPlugin);
// ModelManager 会自动注册为组件
```

### 配置项 (fusion.db.\*)

```toml
[fusion.db]
# PostgreSQL
# url = "postgres://user:password@localhost:5432/dbname"

# 或分别配置
host = "localhost"
port = 5432
username = "postgres"
password = "password"
database = "fusion"

# 连接池
pool_size = 10

# 超时 (秒)
connect_timeout = 30
idle_timeout = 600
```

---

## ModelManager

### 获取实例

```rust
use ultimate_db::ModelManager;
use ultimate_core::Application;

let mm: ModelManager = Application::global().component();
```

### 核心方法

```rust
impl ModelManager {
  pub fn dbx(&self) -> &Dbx;           // 获取数据库执行器

  pub async fn ctx(&self) -> Ctx;      // 获取当前上下文

  pub fn get_txn_clone(&self) -> Self; // 获取事务克隆

  pub fn with_ctx(&self, ctx: Ctx) -> Self; // 设置上下文
}
```

### 上下文管理

```rust
let mm = mm.with_ctx(ctx);  // 设置 Ctx

let ctx = mm.ctx().await;   // 获取 Ctx
```

### 事务处理

```rust
let mm_txn = mm.get_txn_clone();

let ctx = mm_txn.ctx().await;
mm_txn.dbx().begin_txn().await?;

// 执行操作...
UserBmc::create(&mm_txn, input).await?;

// 提交或回滚
match result {
  Ok(_) => mm_txn.dbx().commit_txn().await?,
  Err(e) => { mm_txn.dbx().rollback_txn().await?; return Err(e); }
}
```

---

## fusionsql 集成

fusion-db 重导出 fusionsql 的核心类型：

```rust
use ultimate_db::DbConfig;
use fusionsql::{ModelManager, DbBmc, BmcConfig, SqlError};
```

### DbConfig

```rust
#[derive(Debug, Clone)]
pub struct DbConfig {
  pub url: String,
  pub pool_size: u32,
  pub connect_timeout: u64,
  pub idle_timeout: u64,
}
```

---

## 使用模式

### 完整应用启动流程

```rust
use ultimate_core::{Application, ApplicationBuilder, logforth::LogforthPlugin};
use ultimate_db::DbPlugin;
use ultimate_web::server::WebServerBuilder;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  let app = Application::builder()
    .add_plugin(LogforthPlugin)
    .add_plugin(DbPlugin)
    .run()
    .await?;

  let mm: ModelManager = app.component();

  // ... 业务逻辑

  Ok(())
}
```

---

**版本**: fusion-db v0.1.0+
